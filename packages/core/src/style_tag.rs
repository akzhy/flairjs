use oxc::ast::ast::{
  JSXAttributeItem, JSXAttributeValue, JSXChild, JSXElement, JSXElementName, JSXExpression,
};
use oxc::{
  ast::ast::JSXAttributeName,
  ast_visit::{walk, Visit},
  semantic::{Scoping, SymbolId},
};

use crate::transform::CSSData;

pub struct StyleDetector<'a> {
  scoping: &'a Scoping,
  /// Vector of symbol IDs that represent imported style tag components
  /// eg import { Style } from "@flairjs/react"
  style_tag_import_symbols: &'a Vec<SymbolId>,
  /// Collection of span start positions for detected style tag elements
  /// Used to track where style elements are located in the source code
  /// This will be used to delete the style elements after extraction
  style_tag_symbol_ids: Vec<u32>,
  /// Extracted CSS data from all detected style elements
  pub css: Vec<CSSData>,
  /// Flag indicating whether any style elements were found during traversal
  pub has_style: bool,
}

impl StyleDetector<'_> {
  pub fn new<'a>(
    scoping: &'a Scoping,
    style_tag_import_symbols: &'a Vec<SymbolId>,
  ) -> StyleDetector<'a> {
    let has_style = false;
    let css = vec![];
    let style_tag_symbol_ids = vec![];

    StyleDetector {
      has_style,
      css,
      scoping,
      style_tag_import_symbols,
      style_tag_symbol_ids,
    }
  }

  pub fn get_style_tag_symbol_ids(&self) -> Vec<u32> {
    self.style_tag_symbol_ids.to_vec()
  }
}

impl<'a> Visit<'_> for StyleDetector<'a> {
  fn visit_jsx_element(&mut self, jsx: &JSXElement<'_>) {
    let name = &jsx.opening_element.name;

    if let JSXElementName::IdentifierReference(ident) = name {
      // Resolve the identifier to its symbol using the scoping context
      let reference = self.scoping.get_reference(ident.reference_id());
      let symbol_id = reference.symbol_id().unwrap();

      // Check if this symbol matches any of the imported style tag symbols
      if self.style_tag_import_symbols.contains(&symbol_id) {
        self.has_style = true;

        // Store the span start position for this style element
        // This helps track where the styled component appears in source code so it can be removed later
        self.style_tag_symbol_ids.push(jsx.span.start);

        let children_iter = jsx.children.iter();

        let mut extracted_css: String = "".to_string();

        // Check if this style element should be treated as global CSS
        let is_global = check_if_global(jsx);

        // Extract CSS content from the children of the styled component
        for child in children_iter {
          // Handle direct text content (e.g., <Style>body { color: red; }</Style>)
          if let JSXChild::Text(child_text) = child {
            extracted_css.push_str(&child_text.value);
          }
          // Handle JavaScript expressions containing CSS (e.g., <Style>{`body { color: red; }`}</Style>)
          else if let JSXChild::ExpressionContainer(child_expression) = child {
            let expression = &child_expression.expression;
            if let JSXExpression::TemplateLiteral(template_expression) = expression {
              // Extract the raw string content from template literal quasi elements
              // Note: This only extracts static parts, not interpolated expressions
              let template_expression_value = template_expression
                .quasis
                .iter()
                .map(|elem| elem.value.clone().raw.into_string())
                .collect::<Vec<String>>()
                .join("");

              extracted_css.push_str(&template_expression_value);
            } else if let JSXExpression::TaggedTemplateExpression(tagged_template) = expression {
              // Handle tagged template literals (e.g., css`body { color: red; }`)
              let tagged_template_value = tagged_template
                .quasi
                .quasis
                .iter()
                .map(|elem| elem.value.clone().raw.into_string())
                .collect::<Vec<String>>()
                .join("");

              extracted_css.push_str(&tagged_template_value);
            }
          }
        }
        self.css.push(CSSData {
          raw_css: extracted_css,
          is_global,
        });
      }
    }

    walk::walk_jsx_element(self, jsx);
  }
}

/// Determines if a JSX style element should be treated as global CSS
/// by checking for the presence and value of a "global" attribute.
///
/// Examples:
/// - `<Style global>` or `<Style global={true}>` returns true
/// - `<Style global={false}>` returns false  
/// - `<Style>` (no global attribute) returns false
fn check_if_global(jsx: &JSXElement) -> bool {
  // Use any() to check if any attribute matches our global criteria
  jsx.opening_element.attributes.iter().any(|attr_item| {
    match attr_item {
      // Only consider actual attributes, not spreads
      JSXAttributeItem::Attribute(attr) => {
        match &attr.name {
          // Look for an attribute specifically named "global"
          JSXAttributeName::Identifier(ident) if ident.name == "global" => {
            match &attr.value {
              // Handle <Style global={expression}> case
              Some(JSXAttributeValue::ExpressionContainer(expr)) => {
                // Check if the expression is a boolean literal with value true
                matches!(&expr.expression, JSXExpression::BooleanLiteral(bool_lit) if bool_lit.value)
              }
              // Handle <Style global> case (attribute present without explicit value)
              // In JSX, this is equivalent to global={true}
              None => true,
              // Handle other value types (strings, etc.) - treat as false
              _ => false,
            }
          }
          // Ignore attributes that aren't named "global"
          _ => false,
        }
      }
      // Ignore spread attributes
      _ => false,
    }
  })
}
