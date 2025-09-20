use std::{collections::HashMap, convert::Infallible};

use cssparser::{ParseError, Parser, ParserInput, SourceLocation, ToCss, Token};
use lightningcss::{
  css_modules::{self},
  printer::PrinterOptions,
  stylesheet::{ParserOptions, StyleSheet, ToCssResult},
  targets::{Browsers, Features, Targets},
};

use crate::{log_error, transform::Theme};

/// Parses CSS string and applies transformations based on configuration flags
///
/// # Arguments
/// * `css` - Raw CSS string to parse
/// * `filename` - Name of the file being parsed (used for error reporting and source maps)
/// * `module` - Whether to enable CSS modules (scoped class names)
/// * `use_theme` - Whether to process theme tokens (e.g., $theme.color.primary -> var(--theme-color-primary))
///
/// # Returns
/// * `Ok(ToCssResult)` - Parsed and transformed CSS with optional exports (for CSS modules)
/// * `Err(String)` - Error message if parsing or transformation fails
pub fn parse_css(
  css: &str,
  filename: &str,
  module: bool,
  use_theme: bool,
  theme: &Option<Theme>,
) -> Result<ToCssResult, String> {
  // Pre-process CSS to replace theme tokens if enabled
  // Theme tokens like $theme.color.primary get converted to var(--theme-color-primary)
  let processed_css = if use_theme {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    replace_theme_tokens(&mut parser, theme)
  } else {
    css.to_string()
  };

  // Configure parser options for lightningcss
  let parser_options = ParserOptions {
    filename: filename.to_string(),
    // Enable CSS modules if requested - this will scope class names and generate export mappings
    css_modules: if module {
      Some(css_modules::Config {
        ..Default::default()
      })
    } else {
      None
    },
    ..Default::default()
  };

  // Set up browser targets for CSS transformations
  // "defaults" refers to browserslist's default query (last 2 versions, >0.2% usage, not dead)
  let browsers = Browsers::from_browserslist(vec!["defaults"]).unwrap();
  let targets = Targets {
    browsers,
    // Enable CSS nesting support in addition to default features
    // This allows nested selectors to be processed and flattened if needed for older browsers
    include: Features::default() | Features::Nesting,
    ..Targets::default()
  };

  let stylesheet = StyleSheet::parse(&processed_css, parser_options)
    .map_err(|e| format!("Failed to parse CSS: {}", e))?;

  // Convert the stylesheet back to CSS string with transformations applied
  let result = stylesheet.to_css(PrinterOptions {
    minify: false, // Expect the users' bundler to handle minification
    targets,
    ..Default::default()
  });

  // Handle the conversion result and provide descriptive error messages
  let ret_value = match result {
    Ok(result) => result,
    Err(e) => return Err(format!("Failed to convert stylesheet to CSS: {}", e)),
  };
  Ok(ret_value)
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

  let breakpoints = if let Some(theme) = theme {
    &theme.breakpoints
  } else {
    &HashMap::new()
  };

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
          if let Some(last_var_location) = last_variable_location {
            // We were tracking a potential theme variable, now we need to decide what to do

            // Build fallback string from collected tokens in case theme parsing fails
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
          }
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
    let raw_theme_token = &parser.current_line().to_string()
      [(var_start_location.column - 1) as usize..(last_var_token_location.column - 1) as usize];

    // Convert theme token to CSS custom property with validation
    // Examples:
    // - "$primary" -> "var(--primary)"
    // - "$colors.red.500" -> "var(--colors-red-500)"
    // - "$spaces.4" -> "var(--spaces-4)"
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
    if breakpoints.contains_key(rule.trim()) {
      out.push_str(&format!(
        "@media (min-width: {})",
        breakpoints.get(rule.trim()).unwrap()
      ));
    } else {
      log_error!(
        "Warning: No matching breakpoint found for '@screen {}'",
        rule.trim()
      );
      out.push_str(&fallback_string);
    }
  } else {
    out.push_str(&fallback_string);
  }

  out
}
