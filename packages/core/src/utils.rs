use lightningcss::error::{Error, ErrorLocation, ParserError, PrinterErrorKind};
use napi_derive::napi;
use oxc_data_structures::rope::Rope;

use crate::transform::CSSData;

pub struct ExtendedRope<'a> {
  rope: Rope,
  source_text: &'a str,
}

impl<'a> ExtendedRope<'a> {
  pub fn new(source_text: &'a str) -> Self {
    let rope = Rope::from_str(source_text);
    Self { rope, source_text }
  }

  pub fn get_line_column(&self, offset: u32) -> (u32, u32) {
    let offset = offset as usize;
    // Get line number and byte offset of start of line
    let line_index = self.rope.byte_to_line(offset);
    let line_offset = self.rope.line_to_byte(line_index);
    // Get column number
    let column_index = self.source_text[line_offset..offset].encode_utf16().count();
    ((line_index + 1) as u32, (column_index + 1) as u32)
  }
}

#[napi(object)]
pub struct UnusedCss {
  pub class_name: String,
  pub line: u32,
  pub column: u32,
}

pub fn map_lightning_css_parser_error_to_string(error: Error<ParserError>) -> String {
  match error.kind {
    ParserError::AtRuleBodyInvalid => "At-rule body is invalid".to_string(),
    ParserError::AtRulePreludeInvalid => "At-rule prelude is invalid".to_string(),
    ParserError::AtRuleInvalid(rule_name) => {
      format!("Unknown or unsupported at-rule: {}", rule_name)
    }
    ParserError::EndOfInput => "Unexpected end of input".to_string(),
    ParserError::InvalidDeclaration => "Invalid declaration".to_string(),
    ParserError::InvalidMediaQuery => "Invalid media query".to_string(),
    ParserError::InvalidNesting => "Invalid CSS nesting".to_string(),
    ParserError::DeprecatedNestRule => "The @nest rule is deprecated".to_string(),
    ParserError::DeprecatedCssModulesValueRule => {
      "The @value rule (CSS modules) is deprecated".to_string()
    }
    ParserError::InvalidPageSelector => "Invalid selector in @page rule".to_string(),
    ParserError::InvalidValue => "Invalid value encountered".to_string(),
    ParserError::QualifiedRuleInvalid => "Invalid qualified rule".to_string(),
    ParserError::SelectorError(selector_err) => format!("Selector error: {:?}", selector_err),
    ParserError::UnexpectedImportRule => {
      "@import rule must come before all rules except @charset or @layer".to_string()
    }
    ParserError::UnexpectedNamespaceRule => {
      "@namespace rule must come before all rules except @charset, @import, or @layer".to_string()
    }
    ParserError::UnexpectedToken(token) => format!("Unexpected token: {:?}", token),
    ParserError::MaximumNestingDepth => "Maximum nesting depth reached".to_string(),
  }
}

pub fn map_lightning_css_printer_error_to_string(error: Error<PrinterErrorKind>) -> String {
  match error.kind {
    PrinterErrorKind::AmbiguousUrlInCustomProperty { url } => {
      format!("Ambiguous relative url() in custom property: {}", url)
    }
    PrinterErrorKind::FmtError => "Formatting error occurred".to_string(),
    PrinterErrorKind::InvalidComposesNesting => {
      "CSS modules 'composes' property cannot be used within nested rules".to_string()
    }
    PrinterErrorKind::InvalidComposesSelector => {
      "CSS modules 'composes' property can only be used with a simple class selector".to_string()
    }
    PrinterErrorKind::InvalidCssModulesPatternInGrid => {
      "CSS modules pattern must end with [local] for use in CSS grid".to_string()
    }
  }
}

fn format_file_info(loc: &Option<ErrorLocation>, filename: &str, css_data: &CSSData) -> String {
  let line_number = if let Some(loc) = &loc {
    css_data.line_number + loc.line
  } else {
    css_data.line_number
  };
  let column_number = if let Some(loc) = &loc {
    if loc.line == 0 {
      css_data.column_number + loc.column
    } else {
      loc.column
    }
  } else {
    css_data.column_number
  };

  format!("{}:{}:{}", filename, line_number, column_number)
}

pub trait MapLightningCssError {
  fn to_flair_error_string(self, filename: &str, css_data: &CSSData) -> String;
}

impl<'a> MapLightningCssError for Error<ParserError<'a>> {
  fn to_flair_error_string(self, filename: &str, css_data: &CSSData) -> String {
    let file_info = format_file_info(&self.loc, filename, css_data);
    format!(
      "{} {}",
      file_info,
      map_lightning_css_parser_error_to_string(self)
    )
  }
}

impl MapLightningCssError for Error<PrinterErrorKind> {
  fn to_flair_error_string(self, filename: &str, css_data: &CSSData) -> String {
    let file_info = format_file_info(&self.loc, filename, css_data);
    format!(
      "{} {}",
      file_info,
      map_lightning_css_printer_error_to_string(self)
    )
  }
}
