use std::collections::HashMap;

use lightningcss::css_modules::CssModuleExport;
use oxc::ast_visit::VisitMut;
use oxc_allocator::{Allocator, Box};
use oxc_ast::{
  ast::{JSXAttributeName, JSXAttributeValue, StringLiteral},
  AstBuilder,
};
use oxc_span::Atom;

pub struct AttributeUpdater<'a, 'b> {
  pub class_name_map: HashMap<String, CssModuleExport>,
  pub allocator: &'a Allocator,
  pub ast_builder: AstBuilder<'a>,
  pub phantom: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b> VisitMut<'b> for AttributeUpdater<'a, 'b>
where
  'a: 'b,
{
  fn visit_jsx_attribute(&mut self, it: &mut oxc_ast::ast::JSXAttribute<'b>) {
    if let JSXAttributeName::Identifier(ident) = &it.name {
      if ident.name == "className" {
        let value = it.value.as_ref().unwrap();

        if let JSXAttributeValue::StringLiteral(string_value) = value {
          let class_names: Vec<&str> = string_value.value.split_whitespace().collect();
          let mut updated_class_names = Vec::new();

          for class_name in class_names {
            if let Some(export) = self.class_name_map.get(class_name) {
              updated_class_names.push(export.name.clone());
            } else {
              updated_class_names.push(class_name.to_string());
            }
          }

          let updated_class_names_str = updated_class_names.join(" ");
          let atom = self.allocator.alloc_str(&updated_class_names_str);

          // Create new StringLiteral with proper JSX attribute value
          let new_value =
            self
              .ast_builder
              .jsx_attribute_value_string_literal(string_value.span, atom, None);

          it.value = Some(new_value);
        }
      }
    }
  }
}
