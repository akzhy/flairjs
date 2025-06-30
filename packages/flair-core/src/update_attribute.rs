use std::collections::HashMap;

use lightningcss::css_modules::CssModuleExport;
use oxc::ast_visit::VisitMut;
use oxc_allocator::Allocator;
use oxc_ast::{
  ast::{Expression, JSXAttributeName, JSXAttributeValue, JSXExpression},
  AstBuilder,
};

pub struct AttributeUpdater<'a> {
  pub class_name_map: HashMap<String, CssModuleExport>,
  pub allocator: &'a Allocator,
  pub ast_builder: AstBuilder<'a>,
}

struct UpdaterUtils<'a> {
  pub class_name_map: HashMap<String, CssModuleExport>,
  pub allocator: &'a Allocator,
  pub ast_builder: &'a AstBuilder<'a>,
}

impl UpdaterUtils<'_> {
  fn get_updated_classname(&self, class_name: &str) -> String {
    let class_names: Vec<&str> = class_name.split_whitespace().collect();
    let mut updated_class_names = Vec::new();

    for class_name in class_names {
      if let Some(export) = self.class_name_map.get(class_name) {
        updated_class_names.push(export.name.clone());
      } else {
        updated_class_names.push(class_name.to_string());
      }
    }

    let updated_class_names_str = updated_class_names.join(" ");

    updated_class_names_str
  }
}

impl<'a> VisitMut<'a> for AttributeUpdater<'a> {
  fn visit_jsx_attribute(&mut self, it: &mut oxc_ast::ast::JSXAttribute<'a>) {
    if let JSXAttributeName::Identifier(ident) = &it.name {
      if ident.name == "className" {
        let updater_utils = UpdaterUtils {
          class_name_map: self.class_name_map.clone(),
          allocator: self.allocator,
          ast_builder: &self.ast_builder,
        };

        let value = it.value.as_mut().unwrap();

        if let JSXAttributeValue::StringLiteral(string_value) = value {
          let updated_class_names_str = updater_utils.get_updated_classname(&string_value.value);
          let atom = self.allocator.alloc_str(&updated_class_names_str);

          // Create new StringLiteral with proper JSX attribute value
          let new_value =
            self
              .ast_builder
              .jsx_attribute_value_string_literal(string_value.span, atom, None);

          it.value = Some(new_value);
        } else if let JSXAttributeValue::ExpressionContainer(expr_container) = value {
          let expression = &mut expr_container.expression;
          if let JSXExpression::ArrayExpression(array_expression) = expression {
            array_expression.elements.iter_mut().for_each(|element| {
              if let Some(Expression::StringLiteral(string_value)) = element.as_expression_mut() {
                let updated_class_names_str =
                  updater_utils.get_updated_classname(&string_value.value);
                let atom = self.ast_builder.atom(self.allocator.alloc_str(&updated_class_names_str));

                // Update the string literal value
                string_value.value = atom;
                
              }
            });
          }
          println!(
            "ExpressionContainer found in className attribute: {:#?}",
            expr_container
          );
        }
      }
    }
  }
}
