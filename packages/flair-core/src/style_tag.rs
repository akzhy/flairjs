use oxc::{
  ast_visit::{walk, Visit},
  semantic::{Scoping, SymbolId},
};
use oxc_ast::ast::{JSXChild, JSXElement, JSXElementName, JSXExpression};

pub struct StyleDetector<'a> {
  scoping: &'a Scoping,
  style_tag_symbols: &'a Vec<SymbolId>,
  pub css: Vec<String>,
  pub has_style: bool,
}

impl StyleDetector<'_> {
  pub fn new<'a>(scoping: &'a Scoping, style_tag_symbols: &'a Vec<SymbolId>) -> StyleDetector<'a> {
    let has_style = false;
    let css = vec![];

    StyleDetector {
      has_style,
      css,
      scoping,
      style_tag_symbols,
    }
  }
}

impl<'a> Visit<'_> for StyleDetector<'a> {
  fn visit_jsx_element(&mut self, jsx: &JSXElement<'_>) {
    let name = &jsx.opening_element.name;

    if let JSXElementName::IdentifierReference(ident) = name {
      let reference = self.scoping.get_reference(ident.reference_id());
      let symbol_id = reference.symbol_id().unwrap();
      if self.style_tag_symbols.contains(&symbol_id) {
        self.has_style = true;

        let children_iter = jsx.children.iter();

        let mut extracted_css: String = "".to_string();

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
        self.css.push(extracted_css);
      }
    }

    walk::walk_jsx_element(self, jsx);
  }
}
