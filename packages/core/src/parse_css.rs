use std::{convert::Infallible};

use cssparser::{ParseError, Parser, ParserInput, SourceLocation, ToCss, Token};
use lightningcss::{
  css_modules::{self},
  printer::PrinterOptions,
  stylesheet::{ParserOptions, StyleSheet, ToCssResult},
  targets::{Browsers, Features, Targets},
};

pub fn parse_css(css: &str, filename: &str, module: bool, use_theme: bool) -> Result<ToCssResult, String> {
  let processed_css = if use_theme {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    replace_theme_tokens(&mut parser)
  } else {
    css.to_string()
  };

  let parser_options = ParserOptions {
    filename: filename.to_string(),
    css_modules: if module {
      Some(css_modules::Config {
        ..Default::default()
      })
    } else {
      None
    },
    ..Default::default()
  };

  let browsers = Browsers::from_browserslist(vec!["defaults"]).unwrap();
  let targets = Targets {
    browsers: browsers,
    include: Features::default() | Features::Nesting,
    ..Targets::default()
  };


  let stylesheet =
    StyleSheet::parse(&processed_css, parser_options).map_err(|e| format!("Failed to parse CSS: {}", e))?;

  let result = stylesheet.to_css(PrinterOptions {
    minify: false,
    targets: targets,
    ..Default::default()
  });

  let ret_value = match result {
    Ok(result) => result,
    Err(e) => return Err(format!("Failed to convert stylesheet to CSS: {}", e)),
  };
  Ok(ret_value)
}


fn replace_theme_tokens(parser: &mut Parser<'_, '_>) -> String {
  let mut out = String::from("");
 
  let mut last_variable_location: Option<SourceLocation> = None;
  let mut tokens_stack: Vec<Token> = vec![];
 
  while let Ok(token) = parser.next_including_whitespace() {
    let token_clone = token.clone();
    let current_source_location = parser.current_source_location();
 
    match token_clone {
      Token::CurlyBracketBlock
      | Token::Function(_)
      | Token::ParenthesisBlock
      | Token::SquareBracketBlock => {
        out.push_str(&token_clone.to_css_string());
 
        last_variable_location = None;
 
        let closing = match token_clone {
          Token::CurlyBracketBlock => "}",
          Token::ParenthesisBlock => ")",
          Token::Function(_) => ")",
          Token::SquareBracketBlock => "]",
          _ => "",
        };
 
        let _ = parser.parse_nested_block(|block| {
          let block_out = replace_theme_tokens(block);
          out.push_str(&block_out);
          Ok::<(), ParseError<'_, Infallible>>(())
        });
 
        out.push_str(&closing);
      }
      Token::Ident(_) => {
        if last_variable_location.is_some() {
          tokens_stack.push(token_clone);
        } else {
          out.push_str(&token_clone.to_css_string());
        }
      }
      Token::Delim(delim) => {
        if delim.to_string() == "$" {
          last_variable_location = Some(parser.current_source_location());
          tokens_stack.push(token_clone);
        } else if delim.to_string() == "." && last_variable_location.is_some() {
          tokens_stack.push(token_clone);
        } else {
          out.push_str(&delim.to_string());
        }
      }
      _ => {
        if let Some(last_var_location) = last_variable_location {
          let mut fallback_string = String::from("");
 
          while let Some(var_token) = tokens_stack.pop() {
            fallback_string.push_str(&var_token.to_css_string());
          }
 
          if current_source_location.line == last_var_location.line
            && current_source_location.column > last_var_location.column
          {
            let raw_theme_token = &parser.current_line().to_string()[(last_var_location.column - 1)
              as usize
              ..(current_source_location.column - 1) as usize];
            let parsed_token = {
              let path_vec: Vec<&str> = raw_theme_token.split(".").collect();
              format!("var(--{})", path_vec.join("-"))
            };
            out.push_str(&parsed_token);
          } else {
            out.push_str(&fallback_string);
          }
          last_variable_location = None;
        } else {
          out.push_str(&token_clone.to_css_string());
        }
      }
    }
  }
  out
}