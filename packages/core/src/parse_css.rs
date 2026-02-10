use std::{
  collections::HashMap,
  convert::Infallible,
  hash::{DefaultHasher, Hash, Hasher},
};

use cssparser::{ParseError, Parser, ParserInput, SourceLocation, ToCss, Token};
use data_encoding::{Encoding, Specification};
use lazy_static::lazy_static;
use lightningcss::{
  css_modules::{self, Pattern, Segment},
  printer::PrinterOptions,
  stylesheet::{ParserOptions, StyleSheet, ToCssResult},
  targets::{Browsers, Features, Targets},
};
use smallvec::smallvec;

use crate::{
  log_error,
  transform::{CSSData, Theme},
  utils::MapLightningCssError,
};
use parcel_sourcemap::SourceMap;

#[derive(Debug)]
pub struct ParseCssResult {
  pub result: ToCssResult,
  pub source_map: SourceMap,
}

/// Parses CSS string and applies transformations based on configuration flags
///
/// # Arguments
/// * `css` - Raw CSS string to parse
/// * `filename` - Name of the file being parsed (used for error reporting and source maps)
/// * `module` - Whether to enable CSS modules (scoped class names)
/// * `use_theme` - Whether to process theme tokens (e.g., $theme.color.primary -> var(--theme-color-primary))
///
/// # Returns
/// * `Ok(ParseCssResult)` - Parsed and transformed CSS with optional exports (for CSS modules)
/// * `Err(String)` - Error message if parsing or transformation fails
#[allow(clippy::too_many_arguments)]
pub fn parse_css(
  css: &str,
  source_code: &str,
  modulename: &str,
  filename: &str,
  css_data: CSSData,
  module: bool,
  use_theme: bool,
  theme: &Option<Theme>,
) -> Result<ParseCssResult, String> {
  // Pre-process CSS to replace theme tokens if enabled
  // Theme tokens like $theme.color.primary get converted to var(--theme-color-primary)
  let processed_css = if use_theme {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    replace_theme_tokens(&mut parser, theme)
  } else {
    css.to_string()
  };

  let module_hash_name = hash(modulename, true);
  // Configure parser options for lightningcss
  let parser_options = ParserOptions {
    filename: filename.to_string(),

    // Enable CSS modules if requested - this will scope class names and generate export mappings
    css_modules: if module {
      Some(css_modules::Config {
        pattern: Pattern {
          segments: smallvec![
            Segment::Literal(&module_hash_name),
            Segment::Literal("_"),
            Segment::Local
          ],
        },
        ..Default::default()
      })
    } else {
      None
    },
    ..Default::default()
  };

  // Set up browser targets for CSS transformations
  // "defaults" refers to browserslist's default query (last 2 versions, >0.2% usage, not dead)
  let browsers = Browsers::from_browserslist(vec!["defaults"]).unwrap_or(None);
  let targets = Targets {
    browsers,
    // Enable CSS nesting support in addition to default features
    // This allows nested selectors to be processed and flattened if needed for older browsers
    include: Features::default() | Features::Nesting | Features::MediaRangeSyntax,
    ..Targets::default()
  };

  let stylesheet = StyleSheet::parse(&processed_css, parser_options)
    .map_err(|e| e.to_flair_error_string(filename.to_string(), &css_data))?;

  let project_root = ".";
  let mut sourcemap = SourceMap::new(project_root);
  sourcemap.add_source(filename);

  let _ = sourcemap.set_source_content(0, source_code);
  // Convert the stylesheet back to CSS string with transformations applied
  let result = stylesheet.to_css(PrinterOptions {
    minify: false, // Expect the users' bundler to handle minification
    targets,
    source_map: Some(&mut sourcemap),
    ..Default::default()
  });

  let mut offsetted_sourcemap = SourceMap::new(project_root);
  offsetted_sourcemap.add_source(filename);

  let _ = offsetted_sourcemap.set_source_content(0, source_code);

  sourcemap
    .get_mappings()
    .iter()
    .enumerate()
    .for_each(|(index, map)| {
      let new_original = map.original.map(|mut o| {
        o.original_line += css_data.line_number;

        if index == 0 {
          o.original_column += css_data.column_number;
        }
        o
      });

      offsetted_sourcemap.add_mapping(map.generated_line, map.generated_column, new_original);
    });

  // Handle the conversion result and provide descriptive error messages
  let ret_value = match result {
    Ok(result) => result,
    Err(e) => return Err(e.to_flair_error_string(filename.to_string(), &css_data)),
  };

  Ok(ParseCssResult {
    result: ret_value,
    source_map: offsetted_sourcemap,
  })
}

/// Validates a theme token string to ensure it follows the expected format
///
/// Valid formats:
/// - `$identifier` (e.g., `$primary`)
/// - `$identifier.segment` (e.g., `$colors.red`)
/// - `$identifier.segment.number` (e.g., `$colors.red.500`)
///
/// Note: camelCase identifiers are recommended (e.g., `$primaryColor`) but not enforced
///
/// # Arguments
/// * `token` - The raw theme token string to validate
///
/// # Returns
/// * `true` if the token is valid, `false` otherwise
fn is_valid_theme_token(token: &str) -> bool {
  // Must have at least one character after $
  if token.is_empty() {
    return false;
  }

  // Split by dots and validate each segment
  let segments: Vec<&str> = token.split('.').collect(); // Skip the $ prefix

  for segment in segments {
    if segment.is_empty() {
      return false; // Empty segments like $colors..red are invalid
    }

    // Each segment should be a valid identifier or number
    // Allow alphanumeric, underscore, hyphen, and pure numbers
    // Note: camelCase is recommended but not enforced
    if !segment
      .chars()
      .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
      return false;
    }
  }

  true
}

/// Replaces theme tokens in CSS with CSS custom properties
///
/// Transforms syntax like `$theme.color.primary` into `var(--theme-color-primary)`
/// This function uses a stateful parser to track when we're inside a theme token
/// and accumulates tokens until we reach a non-theme token or line break
///
/// # Arguments
/// * `parser` - CSS parser positioned at the start of the content to process
///
/// # Returns
/// * `String` - CSS with theme tokens replaced by CSS custom properties
fn replace_theme_tokens(parser: &mut Parser<'_, '_>, theme: &Option<Theme>) -> String {
  let mut out = String::from("");

  // Track the location where a potential theme variable started (after seeing '$')
  let mut last_variable_location: Option<SourceLocation> = None;

  let mut last_screen_at_rule_location: Option<SourceLocation> = None;
  // Stack to accumulate tokens that might be part of a theme variable
  let mut tokens_stack: Vec<(Token, SourceLocation)> = vec![];

  let default_breakpoints = HashMap::new();
  let breakpoints = theme
    .as_ref()
    .and_then(|t| t.breakpoints.as_ref())
    .unwrap_or(&default_breakpoints);

  while let Ok(token) = parser.next_including_whitespace() {
    let token_clone = token.clone();

    match token_clone {
      Token::CurlyBracketBlock
      | Token::Function(_)
      | Token::ParenthesisBlock
      | Token::SquareBracketBlock => {
        if let Token::CurlyBracketBlock = token_clone {
          if let Some(last_screen_at_rule) = last_screen_at_rule_location {
            let rule_out =
              handle_at_rule_tokens(parser, &mut tokens_stack, last_screen_at_rule, breakpoints);
            out.push_str(&rule_out);
          }
        }

        last_screen_at_rule_location = None;
        // Output the opening bracket/function name
        out.push_str(&token_clone.to_css_string());

        // Reset variable tracking since we're entering a new context
        last_variable_location = None;

        // Determine the appropriate closing bracket
        let closing = match token_clone {
          Token::CurlyBracketBlock => "}",
          Token::ParenthesisBlock => ")",
          Token::Function(_) => ")",
          Token::SquareBracketBlock => "]",
          _ => "",
        };

        // Recursively process the contents of the block
        let _ = parser.parse_nested_block(|block| {
          let block_out = replace_theme_tokens(block, theme);
          out.push_str(&block_out);
          Ok::<(), ParseError<'_, Infallible>>(())
        });

        // Output the closing bracket
        out.push_str(closing);
      }
      // Handle identifier tokens (variable names, property names, etc.)
      Token::Ident(_) => {
        if last_variable_location.is_some() || last_screen_at_rule_location.is_some() {
          // We're potentially inside a theme variable, so collect this identifier
          tokens_stack.push((token_clone, parser.current_source_location()));
        } else {
          // Regular identifier, output as-is
          out.push_str(&token_clone.to_css_string());
        }
      }
      // Handle delimiter tokens (operators, punctuation)
      Token::Delim(delim) => {
        if delim.to_string() == "$" {
          // Start of a potential theme variable - record location and start collecting tokens
          last_variable_location = Some(parser.current_source_location());
          tokens_stack.push((token_clone, parser.current_source_location()));
        } else if delim.to_string() == "." && last_variable_location.is_some() {
          // Dot within a theme variable (e.g., theme.color.primary) - collect it
          tokens_stack.push((token_clone, parser.current_source_location()));
        } else {
          // Regular delimiter, output as-is
          out.push(delim);
        }
      }
      Token::AtKeyword(ref at_string) => {
        if *at_string == "screen" {
          last_screen_at_rule_location = Some(parser.current_source_location());
          tokens_stack.push((token_clone, parser.current_source_location()));
        } else {
          last_screen_at_rule_location = None;
          out.push_str(&token_clone.to_css_string());
        }
      }
      Token::Dimension { .. } => {
        if last_variable_location.is_some() {
          // Handle cases like $fontSize.3xl where 3xl is a dimension
          tokens_stack.push((token_clone, parser.current_source_location()));
        } else if last_screen_at_rule_location.is_some() {
          tokens_stack.push((token_clone, parser.current_source_location()));
        } else {
          out.push_str(&token_clone.to_css_string());
        }
      }
      Token::Number { .. } => {
        if last_variable_location.is_some() {
          // We're potentially inside a theme variable, so collect this number
          tokens_stack.push((token_clone, parser.current_source_location()));
        } else if last_screen_at_rule_location.is_some() {
          tokens_stack.push((token_clone, parser.current_source_location()));
        } else {
          out.push_str(&token_clone.to_css_string());
        }
      }
      Token::WhiteSpace(white_space) => {
        if last_screen_at_rule_location.is_some() {
          tokens_stack.push((token_clone, parser.current_source_location()));
        } else if let Some(last_var_location) = last_variable_location {
          let theme_out = handle_theme_tokens(
            parser,
            &token_clone,
            &mut tokens_stack,
            last_var_location,
            theme,
          );
          out.push_str(&theme_out);
          // Reset variable tracking
          last_variable_location = None;
        } else {
          out.push_str(white_space);
        }
      }
      // Handle all other token types
      _ => {
        if let Some(last_var_location) = last_variable_location {
          let theme_out = handle_theme_tokens(
            parser,
            &token_clone,
            &mut tokens_stack,
            last_var_location,
            theme,
          );
          out.push_str(&theme_out);
          // Reset variable tracking
          last_variable_location = None;
        } else {
          // No active theme variable tracking, output token as-is
          out.push_str(&token_clone.to_css_string());
        }

        last_screen_at_rule_location = None;
      }
    }
  }
  out
}

/// Processes collected tokens that may form a theme variable and converts them to CSS custom properties
/// If the tokens do not form a valid theme variable, outputs them as-is
fn handle_theme_tokens(
  parser: &Parser<'_, '_>,
  current_token: &Token,
  tokens_stack: &mut Vec<(Token, SourceLocation)>,
  var_start_location: SourceLocation,
  theme: &Option<Theme>,
) -> String {
  let mut out = String::from("");
  let mut fallback_string = String::from("");

  let token_prefix = if let Some(theme) = theme {
    match &theme.prefix {
      Some(prefix) => format!("{}-", prefix),
      None => String::from(""),
    }
  } else {
    String::from("")
  };

  let last_var_location = tokens_stack.last().map(|(_, loc)| *loc);

  while let Some((var_token, _)) = tokens_stack.pop() {
    fallback_string.push_str(&var_token.to_css_string());
  }

  let Some(last_var_token_location) = last_var_location else {
    // No valid variable location found, use fallback
    out.push_str(&fallback_string);
    return out;
  };

  // Check if the current token is on the same line as the variable start
  // This ensures we only process theme variables that are on a single line
  if last_var_token_location.line == var_start_location.line
    && last_var_token_location.column > var_start_location.column
  {
    // Extract the raw theme token text from the current line using column positions
    // This approach is necessary because the CSS parser converts numeric segments
    // like ".500" to Number(0.5), but we need the original "$colors.red.500" syntax
    //
    // POTENTIAL ISSUE: String slicing uses byte offsets while parser columns are
    // character-based. This could cause issues with multi-byte Unicode characters
    // in CSS comments or strings, but should be fine for ASCII theme tokens
    let current_line = parser.current_line().to_string();
    let start_idx = (var_start_location.column - 1) as usize;
    let end_idx = (last_var_token_location.column - 1) as usize;

    // Safely slice the string to avoid panics from out-of-bounds access
    let raw_theme_token_opt = current_line.get(start_idx..end_idx);

    // Convert theme token to CSS custom property with validation
    // Examples:
    // - "$primary" -> "var(--primary)"
    // - "$colors.red.500" -> "var(--colors-red-500)"
    // - "$spaces.4" -> "var(--spaces-4)"
    if let Some(raw_theme_token) = raw_theme_token_opt {
      let parsed_token = if is_valid_theme_token(raw_theme_token) {
        let path_vec: Vec<&str> = raw_theme_token.split(".").collect();
        format!("var(--{token_prefix}{})", path_vec.join("-"))
      } else {
        // Invalid theme token format - log warning and output as fallback
        log_error!("Warning: Invalid theme token format '{}'. Expected format: $identifier or $identifier.segment.value (camelCase recommended)", raw_theme_token);
        // This preserves the original token in case of malformed syntax
        fallback_string.clone()
      };
      out.push_str(&parsed_token);
    } else {
      // Theme variable spans multiple lines or whitespace was encountered
      // Since theme tokens are expected to be single-line expressions,
      // fall back to outputting the original token sequence
      out.push_str(&fallback_string);
    }
  } else {
    // Current token is on a different line than the variable start, so we can't form a valid theme variable
    // Output the collected tokens as-is
    out.push_str(&fallback_string);
  }

  out.push_str(&current_token.to_css_string());

  out
}

/// Handles @screen at-rules by converting them to @media queries based on theme breakpoints
/// Transforms syntax like `@screen md` into `@media (min-width: 768px)`
/// using the breakpoints defined in the theme configuration
/// Requires the theme to be provided with breakpoints
fn handle_at_rule_tokens(
  parser: &Parser<'_, '_>,
  tokens_stack: &mut Vec<(Token, SourceLocation)>,
  screen_at_rule_start_location: SourceLocation,
  breakpoints: &HashMap<String, String>,
) -> String {
  let mut fallback_string = String::from("");
  let mut out = String::from("");

  let last_at_rule_location = tokens_stack.last().map(|(_, loc)| *loc);

  let Some(last_at_rule_location) = last_at_rule_location else {
    // No valid variable location found, use fallback
    out.push_str(&fallback_string);
    return out;
  };

  while let Some((var_token, _)) = tokens_stack.pop() {
    fallback_string.push_str(&var_token.to_css_string());
  }

  // Check if the current token is on the same line as the at-rule start
  // This ensures we only process at-rules that are on a single line
  if last_at_rule_location.line == screen_at_rule_start_location.line
    && last_at_rule_location.column > screen_at_rule_start_location.column
  {
    let rule = &parser.current_line().to_string()[(screen_at_rule_start_location.column - 1)
      as usize
      ..(last_at_rule_location.column - 2) as usize];
    if let Some(breakpoint_value) = breakpoints.get(rule.trim()) {
      out.push_str(&format!("@media (min-width: {})", breakpoint_value));
    } else {
      log_error!(
        "Error: No matching breakpoint found for '@screen {}'",
        rule.trim()
      );
      out.push_str(&fallback_string);
    }
  } else {
    out.push_str(&fallback_string);
  }

  out
}

lazy_static! {
  static ref ENCODER: Encoding = {
    let mut spec = Specification::new();
    spec
      .symbols
      .push_str("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_-");
    spec.encoding().unwrap()
  };
}

/// Hashing implementation copied from lightningcss.
fn hash(s: &str, at_start: bool) -> String {
  let mut hasher = DefaultHasher::new();
  s.hash(&mut hasher);
  let hash = hasher.finish() as u32;

  let hash = ENCODER.encode(&hash.to_le_bytes());
  if at_start && hash.as_bytes()[0].is_ascii_digit() {
    format!("_{}", hash)
  } else {
    hash
  }
}
