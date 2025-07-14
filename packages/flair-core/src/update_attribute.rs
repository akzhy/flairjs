use std::collections::HashMap;

use lightningcss::css_modules::CssModuleExport;
use oxc::{
  ast_visit::VisitMut,
  semantic::{Scoping, SymbolId},
};
use oxc_allocator::Allocator;
use oxc_allocator::Box as OxcBox;
use oxc_ast::{
  ast::{Expression, JSXAttributeName, JSXAttributeValue, JSXExpression, StringLiteral},
  AstBuilder,
};

pub struct AttributeUpdater<'a> {
  pub class_name_map: HashMap<String, CssModuleExport>,
  pub allocator: &'a Allocator,
  pub ast_builder: AstBuilder<'a>,
  pub scoping: &'a Scoping,
  pub identifier_symbol_ids: Vec<SymbolId>,
}

impl<'a> AttributeUpdater<'a> {
  fn get_classname_exports(&self) -> &HashMap<String, CssModuleExport> {
    &self.class_name_map
  }

  pub fn get_identifier_symbol_ids(&self) -> &Vec<SymbolId> {
    &self.identifier_symbol_ids
  }

  fn get_updated_classname(&self, class_name: &str) -> String {
    let class_names: Vec<&str> = class_name.split_whitespace().collect();
    let mut updated_class_names = Vec::new();

    for class_name in class_names {
      if let Some(export) = self.get_classname_exports().get(class_name) {
        updated_class_names.push(export.name.clone());
      } else {
        updated_class_names.push(class_name.to_string());
      }
    }

    let updated_class_names_str = updated_class_names.join(" ");

    updated_class_names_str
  }

  fn update_string_expression(&self, string_value: &mut OxcBox<'a, StringLiteral<'a>>) {
    let updated_class_names_str = self.get_updated_classname(&string_value.value);
    let atom = self
      .ast_builder
      .atom(self.allocator.alloc_str(&updated_class_names_str));

    // Update the string literal value
    string_value.value = atom;
  }

  fn update_expression(&self, expression: Option<&mut Expression<'a>>) {
    if expression.is_none() {
      return;
    };
    let expression = expression.unwrap();
    match expression {
      Expression::StringLiteral(string_value) => {
        self.update_string_expression(string_value);
      }
      _ => {
        // println!("Unexpected expression type in className attribute",);
      }
    }
  }
}

impl<'a> VisitMut<'a> for AttributeUpdater<'a> {
  fn visit_jsx_attribute(&mut self, it: &mut oxc_ast::ast::JSXAttribute<'a>) {
    if let JSXAttributeName::Identifier(ident) = &it.name {
      if ident.name == "className" {
        let value = it.value.as_mut().unwrap();

        if let JSXAttributeValue::StringLiteral(string_value) = value {
          let updated_class_names_str = self.get_updated_classname(&string_value.value);
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
              self.update_expression(element.as_expression_mut());
            });
          } else if let JSXExpression::CallExpression(call_expression) = expression {
            call_expression.arguments.iter_mut().for_each(|arg| {
              self.update_expression(arg.as_expression_mut());
            });
          } else if let JSXExpression::ConditionalExpression(conditional_expression) = expression {
            {
              let consequent = &mut conditional_expression.consequent;
              self.update_expression(Some(consequent));
            }
            {
              let alternate = &mut conditional_expression.alternate;
              self.update_expression(Some(alternate));
            }
          } else if let JSXExpression::Identifier(identifier_expression) = expression {
            let reference = self
              .scoping
              .get_reference(identifier_expression.reference_id());
            let symbol_id = reference.symbol_id();
            if symbol_id.is_some() {
              println!(
                "Pushing reference name {:#?} with symbol id {:#?}",
                identifier_expression.name,
                symbol_id.unwrap(),
              );
              self.identifier_symbol_ids.push(symbol_id.unwrap());
            }
          } else {
            // println!(
            //   "ExpressionContainer found in className attribute: {:#?}",
            //   expr_container
            // );
          }
        }
      }
    }
  }
}
