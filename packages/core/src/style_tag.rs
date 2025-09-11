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
  style_tag_import_symbols: &'a Vec<SymbolId>,
  style_tag_symbol_ids: Vec<u32>,
  pub css: Vec<CSSData>,
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
      let reference = self.scoping.get_reference(ident.reference_id());
      let symbol_id = reference.symbol_id().unwrap();

      if self.style_tag_import_symbols.contains(&symbol_id) {
        self.has_style = true;

        self.style_tag_symbol_ids.push(jsx.span.start);

        let children_iter = jsx.children.iter();

        let mut extracted_css: String = "".to_string();

        let is_global = check_if_global(jsx);

        for child in children_iter {
          if let JSXChild::Text(child_text) = child {
            extracted_css.push_str(&child_text.value);
          } else if let JSXChild::ExpressionContainer(child_expression) = child {
            let expression = &child_expression.expression;
            if let JSXExpression::TemplateLiteral(template_expression) = expression {
              let template_expression_value = template_expression
                .quasis
                .iter()
                .map(|elem| elem.value.clone().raw.into_string())
                .collect::<Vec<String>>()
                .join("");

              extracted_css.push_str(&template_expression_value);
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

fn check_if_global(jsx: &JSXElement) -> bool {
  jsx.opening_element.attributes.iter().any(|attr_item| {
    match attr_item {
      JSXAttributeItem::Attribute(attr) => {
        match &attr.name {
          JSXAttributeName::Identifier(ident) if ident.name == "global" => {
            match &attr.value {
              Some(JSXAttributeValue::ExpressionContainer(expr)) => {
                matches!(&expr.expression, JSXExpression::BooleanLiteral(bool_lit) if bool_lit.value)
              }
              None => true, // If the attribute is present without a value, treat it as true
              _ => false,
            }
          }
          _ => false,
        }
      }
      _ => false,
    }
  })
}
