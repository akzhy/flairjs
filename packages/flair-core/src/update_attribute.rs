use std::collections::HashMap;

use lightningcss::css_modules::CssModuleExport;
use oxc::{
  ast_visit::{walk_mut, VisitMut},
  semantic::{Scoping, SymbolId},
};
use oxc_allocator::Allocator;
use oxc_allocator::Box as OxcBox;
use oxc_ast::{
  ast::{
    ArrayExpression, BinaryExpression, BinaryOperator, CallExpression, ConditionalExpression,
    Expression, IdentifierReference, JSXAttributeName, JSXAttributeValue, JSXExpression,
    LogicalExpression, LogicalOperator, ObjectExpression, ObjectPropertyKind, PropertyKey,
    StringLiteral,
  },
  AstBuilder,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolStore {
  pub symbol_id: SymbolId,
  pub fn_id: u32,
}

impl SymbolStore {
  pub fn new(symbol_id: SymbolId, fn_id: u32) -> Self {
    Self { symbol_id, fn_id }
  }
}

pub struct ClassNameReplacer<'a> {
  pub class_name_map: HashMap<String, CssModuleExport>,
  pub allocator: &'a Allocator,
  pub ast_builder: AstBuilder<'a>,
  pub scoping: &'a Scoping,
  pub identifier_symbol_ids: Vec<SymbolStore>,
  pub fn_id: u32,
  pub classname_util_symbols: Vec<SymbolId>,
}

impl<'a> ClassNameReplacer<'a> {
  fn get_classname_exports(&self) -> &HashMap<String, CssModuleExport> {
    &self.class_name_map
  }

  pub fn get_identifier_symbol_ids(&self) -> &Vec<SymbolStore> {
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

  fn update_string_expression(&mut self, string_value: &mut OxcBox<'a, StringLiteral<'a>>) {
    let updated_class_names_str = self.get_updated_classname(&string_value.value);
    let atom = self
      .ast_builder
      .atom(self.allocator.alloc_str(&updated_class_names_str));

    // Update the string literal value
    string_value.value = atom;
  }

  fn update_array_expression(&mut self, array_expression: &mut OxcBox<'a, ArrayExpression<'a>>) {
    array_expression.elements.iter_mut().for_each(|element| {
      self.update_expression(element.as_expression_mut());
    });
  }

  fn update_object_expression(&mut self, object_expression: &mut OxcBox<'a, ObjectExpression<'a>>) {
    for prop in &mut object_expression.properties {
      if let ObjectPropertyKind::ObjectProperty(property) = prop {
        if let PropertyKey::StringLiteral(string_key) = &mut property.key {
          self.update_string_expression(string_key);
        }
      }
    }
  }

  fn update_call_expression(&mut self, call_expression: &mut OxcBox<'a, CallExpression<'a>>) {
    call_expression.arguments.iter_mut().for_each(|arg| {
      self.update_expression(arg.as_expression_mut());
    });
  }

  fn update_conditional_expression(
    &mut self,
    conditional_expression: &mut OxcBox<'a, ConditionalExpression<'a>>,
  ) {
    self.update_expression(Some(&mut conditional_expression.consequent));
    self.update_expression(Some(&mut conditional_expression.alternate));
  }

  fn update_logical_expression(
    &mut self,
    logical_expression: &mut OxcBox<'a, LogicalExpression<'a>>,
  ) {
    if logical_expression.operator == LogicalOperator::Or
      || logical_expression.operator == LogicalOperator::Coalesce
    {
      self.update_expression(Some(&mut logical_expression.left));
    }
    self.update_expression(Some(&mut logical_expression.right));
  }

  fn update_binary_expression(&mut self, binary_expression: &mut OxcBox<'a, BinaryExpression<'a>>) {
    if BinaryOperator::Addition == binary_expression.operator {
      self.update_expression(Some(&mut binary_expression.left));
      self.update_expression(Some(&mut binary_expression.right));
    }
  }

  fn update_identifier_expression(&mut self, identifier_expression: &mut IdentifierReference<'a>) {
    let reference = self
      .scoping
      .get_reference(identifier_expression.reference_id());
    let symbol_id = reference.symbol_id();
    if symbol_id.is_some() {
      self
        .identifier_symbol_ids
        .push(SymbolStore::new(symbol_id.unwrap(), self.fn_id));
    }
  }

  pub fn update_expression(&mut self, expression: Option<&mut Expression<'a>>) {
    if expression.is_none() {
      return;
    };
    let expression = expression.unwrap();
    match expression {
      Expression::StringLiteral(string_value) => {
        self.update_string_expression(string_value);
      }
      Expression::ObjectExpression(object_expression) => {
        self.update_object_expression(object_expression);
      }
      Expression::ArrayExpression(array_expression) => {
        self.update_array_expression(array_expression);
      }
      Expression::LogicalExpression(logical_expression) => {
        self.update_logical_expression(logical_expression);
      }
      Expression::ConditionalExpression(conditional_expression) => {
        self.update_conditional_expression(conditional_expression);
      }
      Expression::BinaryExpression(binary_expression) => {
        self.update_binary_expression(binary_expression);
      }
      Expression::CallExpression(call_expression) => {
        self.update_call_expression(call_expression);
      }
      Expression::Identifier(identifier_expression) => {
        self.update_identifier_expression(identifier_expression);
      }
      _ => {
        println!(
          "Unexpected expression type in className attribute {:#?}",
          expression
        );
      }
    }
  }
}

impl<'a> VisitMut<'a> for ClassNameReplacer<'a> {
  fn visit_call_expression(&mut self, it: &mut CallExpression<'a>) {
    if let Expression::Identifier(identifier_calle) = &it.callee {
      let callee_ref = self.scoping.get_reference(identifier_calle.reference_id());
      let callee_symbol_id = callee_ref.symbol_id();
      if callee_symbol_id.is_some() {
        let callee_symbol_id = callee_symbol_id.unwrap();
        if self.classname_util_symbols.contains(&callee_symbol_id) {
          // If the callee is a classname utility function, we can update the arguments
          it.arguments.iter_mut().for_each(|arg| {
            self.update_expression(arg.as_expression_mut());
          });
        }
      }
    }
  }
  fn visit_jsx_attribute(&mut self, it: &mut oxc_ast::ast::JSXAttribute<'a>) {
    if let JSXAttributeName::Identifier(ident) = &it.name {
      if ident.name == "className" {
        let value = it.value.as_mut().unwrap();

        if let JSXAttributeValue::StringLiteral(string_value) = value {
          self.update_string_expression(string_value);
        } else if let JSXAttributeValue::ExpressionContainer(expr_container) = value {
          let expression = &mut expr_container.expression;
          if let JSXExpression::ArrayExpression(array_expression) = expression {
            self.update_array_expression(array_expression);
          } else if let JSXExpression::CallExpression(call_expression) = expression {
            self.update_call_expression(call_expression);
          } else if let JSXExpression::LogicalExpression(logical_expression) = expression {
            self.update_logical_expression(logical_expression);
          } else if let JSXExpression::ConditionalExpression(conditional_expression) = expression {
            self.update_conditional_expression(conditional_expression);
          } else if let JSXExpression::ObjectExpression(object_expression) = expression {
            self.update_object_expression(object_expression);
          } else if let JSXExpression::BinaryExpression(binary_expression) = expression {
            self.update_binary_expression(binary_expression);
          } else if let JSXExpression::Identifier(identifier_expression) = expression {
            self.update_identifier_expression(identifier_expression);
          } else {
            println!(
              "ExpressionContainer found in className attribute: {:#?}",
              expr_container
            );
          }
        }
      }
    }
    walk_mut::walk_jsx_attribute(self, it);
  }
}
