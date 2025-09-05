use oxc::ast_visit::walk;
use oxc::ast_visit::Visit;
use oxc::semantic::ScopeFlags;
use oxc::semantic::Scoping;
use oxc::semantic::SymbolId;
use oxc_ast::ast::BindingPatternKind;
use oxc_ast::ast::Function;
use oxc_ast::ast::VariableDeclaration;
use oxc_ast::ast::{AssignmentTarget, Expression};
use std::collections::HashMap;

pub struct FlairProperty<'a> {
  scoping: &'a Scoping,
  style: HashMap<u32, String>,
  symbol_to_span_start: HashMap<SymbolId, u32>,
}

impl<'a> FlairProperty<'a> {
  pub fn new(scoping: &'a Scoping) -> FlairProperty<'a> {
    FlairProperty {
      scoping,
      style: HashMap::new(),
      symbol_to_span_start: HashMap::new(),
    }
  }

  pub fn get_style(&self) -> &HashMap<u32, String> {
    &self.style
  }
}

impl<'a> Visit<'a> for FlairProperty<'a> {
  fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
    it.declarations.iter().for_each(|decl| {
      if let Some(init) = &decl.init {
        if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
          let item = get_item(init);

          if let Some(span_start) = item {
            self
              .symbol_to_span_start
              .insert(ident.symbol_id(), span_start);
          }
        }
      }
    });

    walk::walk_variable_declaration(self, it)
  }

  fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
    if let Some(name) = &it.id {
      self
        .symbol_to_span_start
        .insert(name.symbol_id(), it.span.start);
    }

    walk::walk_function(self, it, flags);
  }

  fn visit_expression(&mut self, it: &Expression<'a>) {
    let Expression::AssignmentExpression(assign) = it else {
      return walk::walk_expression(self, it);
    };

    let AssignmentTarget::StaticMemberExpression(static_member) = &assign.left else {
      return walk::walk_expression(self, it);
    };

    let Expression::Identifier(ident) = &static_member.object else {
      return walk::walk_expression(self, it);
    };

    let reference = ident.reference_id();
    let symbol_id = self.scoping.get_reference(reference).symbol_id().unwrap();

    if !self.symbol_to_span_start.contains_key(&symbol_id)
      || static_member.property.name.as_str() != "flair"
    {
      return walk::walk_expression(self, it);
    }

    let content = &assign.right;
    let extracted_css = match content {
      Expression::StringLiteral(string_value) => string_value.value.to_string(),
      Expression::TemplateLiteral(template_expression) => {
        let template_value = template_expression
          .quasis
          .iter()
          .map(|elem| elem.value.clone().raw.into_string())
          .collect::<Vec<String>>()
          .join("");

        template_value
      }
      _ => {
        return walk::walk_expression(self, it);
      }
    };

    self.style.insert(
      self.symbol_to_span_start.get(&symbol_id).unwrap().clone(),
      extracted_css,
    );
    walk::walk_expression(self, it)
  }
}

fn get_item(expression: &Expression) -> Option<u32> {
  match expression {
    Expression::FunctionExpression(fn_expr) => {
      return Some(fn_expr.span.start);
    }
    Expression::ArrowFunctionExpression(arrow_expr) => {
      return Some(arrow_expr.span.start);
    }
    Expression::CallExpression(call_expr) => {
      let result = call_expr.arguments.iter().find_map(|arg| {
        let expr = arg.as_expression();
        if let Some(expr) = expr {
          let val = get_item(expr);
          return val;
        }

        return None;
      });
      result
    }
    _ => None,
  }
}
