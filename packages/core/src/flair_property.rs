use indexmap::IndexMap;
use oxc::allocator::Allocator;
use oxc::allocator::Box as OxcBox;
use oxc::ast::ast::BindingPatternKind;
use oxc::ast::ast::Class;
use oxc::ast::ast::Function;
use oxc::ast::ast::ObjectExpression;
use oxc::ast::ast::ObjectPropertyKind;
use oxc::ast::ast::PropertyKey;
use oxc::ast::ast::StringLiteral;
use oxc::ast::ast::VariableDeclaration;
use oxc::ast::ast::{AssignmentTarget, Expression};
use oxc::ast::AstBuilder;
use oxc::semantic::Scoping;
use oxc::semantic::SymbolId;
use std::collections::HashMap;

use crate::transform::CSSData;

pub static FLAIR_REPLACEMENT: &str = "__flair_replacement__";

pub struct FlairProperty<'a> {
  scoping: &'a Scoping,
  style: IndexMap<u32, CSSData>,
  global_style: IndexMap<u32, CSSData>,
  /// Maps function / variable declaration symbol IDs to their starting span positions.
  /// Used to associate flair styles with the correct function/component.
  /// 
  /// Eg: function App() { ... } -> symbol_id: Symbol id of App -> span_start: starting position of function App
  /// Eg: const App = () => { ... } -> symbol_id: Symbol id of App -> span_start: starting position of arrow function
  /// 
  /// This is used to identify flair properties assigned to components.
  /// For example, if we see `App.flair = ...`, we can look up the symbol ID for `App`
  /// and find the starting span position of the function/arrow assigned to `App`.
  /// This allows us to link the flair style to the correct component.
  /// 
  /// Since oxc directly doesn't provide an id for function/arrow expressions,
  /// we use span.start as a unique identifier for the function/arrow.
  symbol_to_span_start_map: HashMap<SymbolId, u32>,
  allocator: &'a Allocator,
  ast_builder: AstBuilder<'a>,
}

impl<'a> FlairProperty<'a> {
  pub fn new(scoping: &'a Scoping, allocator: &'a Allocator) -> FlairProperty<'a> {
    FlairProperty {
      scoping,
      style: IndexMap::new(),
      global_style: IndexMap::new(),
      symbol_to_span_start_map: HashMap::new(),
      allocator,
      ast_builder: AstBuilder::new(&allocator),
    }
  }

  pub fn get_scoped_style(&self) -> &IndexMap<u32, CSSData> {
    &self.style
  }

  pub fn get_global_style(&self) -> &IndexMap<u32, CSSData> {
    &self.global_style
  }

  /// Visit variable declarations to find functions assigned to variables
  ///
  /// For example: 
  /// ```
  /// const MyComponent = () => { ... }
  /// const MyComponent = function() { ... }
  /// ```
  /// Associates variable names with their function's starting span position.
  /// This is important for later linking flair styles to the correct function/component.
  pub fn visit_variable_declaration(&mut self, it: &mut VariableDeclaration<'a>) {
    it.declarations.iter().for_each(|decl| {
      if let Some(init) = &decl.init {
        // Only handle simple variable bindings (not destructuring)
        if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
          // get_item returns the span start for function/arrow/call expressions
          let item = get_item(init);

          if let Some(span_start) = item {
            // Map the variable's symbol ID to the function's span start
            self
              .symbol_to_span_start_map
              .insert(ident.symbol_id(), span_start);
          }
        }
      }
    });
  }

  /// Called when entering a class node in the AST.
  /// Sets flags and records the class's symbol ID and span start for later flair association.
  pub fn visit_class(&mut self, it: &Class<'a>) {
    let Some(class_id) = &it.id else {
      return;
    };
    // Map the class symbol ID to its span start
    self
      .symbol_to_span_start_map
      .insert(class_id.symbol_id(), it.span.start);
  }


  pub fn visit_function(&mut self, it: &mut Function<'a>) {
    if let Some(name) = &it.id {
      self
        .symbol_to_span_start_map
        .insert(name.symbol_id(), it.span.start);
    }
  }

  /// Handles assignment expressions like `Component.flair = ...` or `Component.globalFlair = ...`.
  /// Extracts the CSS content from the right-hand side and stores it in the appropriate style map.
  /// Replaces the original assignment with a string literal marker in the AST.
  pub fn visit_expression(&mut self, it: &mut Expression<'a>) {
    // Only process assignment expressions
    let Expression::AssignmentExpression(assign) = it else {
      return;
    };

    // Only handle static member assignments (e.g., Component.flair)
    let AssignmentTarget::StaticMemberExpression(static_member) = &assign.left else {
      return;
    };

    // Only handle assignments to identifiers (not computed properties)
    let Expression::Identifier(ident) = &static_member.object else {
      return;
    };

    // Get the symbol ID for the identifier being assigned to
    let reference = ident.reference_id();
    let symbol_id = self.scoping.get_reference(reference).symbol_id().unwrap();

    // Only process if the symbol is tracked and the property is 'flair' or 'globalFlair'
    if !self.symbol_to_span_start_map.contains_key(&symbol_id)
      || !(static_member.property.name.as_str() == "flair"
        || static_member.property.name.as_str() == "globalFlair")
    {
      return;
    }

    let content = &assign.right;
    // Extract CSS content from the right-hand side expression
    let css_content: String = match content {
      // Direct string assignment
      Expression::StringLiteral(string_value) => string_value.value.to_string(),
      // Template literal assignment
      Expression::TemplateLiteral(template_expression) => {
        let template_value = template_expression
          .quasis
          .iter()
          .map(|elem| elem.value.clone().raw.into_string())
          .collect::<Vec<String>>()
          .join("");

        template_value
      }
      Expression::TaggedTemplateExpression(tagged_template) => {
        // Handle tagged template literals (e.g., css`body { color: red; }`)
        let tagged_template_value = tagged_template
          .quasi
          .quasis
          .iter()
          .map(|elem| elem.value.clone().raw.into_string())
          .collect::<Vec<String>>()
          .join("");

        tagged_template_value
      }
      // Assignment via flair({...}) call
      Expression::CallExpression(call_expr) => {
        match (
          &call_expr.callee,
          call_expr
            .arguments
            .get(0)
            .and_then(|arg| arg.as_expression()),
        ) {
          // Only handle flair({...}) calls
          (Expression::Identifier(identifier_calle), Some(Expression::ObjectExpression(obj)))
            if identifier_calle.name == "flair" =>
          {
            let style = build_style_string_from_object(obj);
            style
          }
          _ => String::from(""),
        }
      }
      // Other types are ignored
      _ => String::from(""),
    };

    // Store the CSS content in the appropriate style map
    if static_member.property.name.as_str() == "globalFlair" {
      self.global_style.insert(
        self
          .symbol_to_span_start_map
          .get(&symbol_id)
          .unwrap()
          .clone(),
        CSSData {
          raw_css: css_content,
          is_global: true,
        },
      );
    } else {
      self.style.insert(
        self
          .symbol_to_span_start_map
          .get(&symbol_id)
          .unwrap()
          .clone(),
        CSSData {
          raw_css: css_content,
          is_global: false,
        },
      );
    }

    // Replace the assignment expression in the AST with a string literal marker
    // This marker will be deleted later in the transformation process
    let atom = self
      .ast_builder
      .atom(self.allocator.alloc_str(&FLAIR_REPLACEMENT));

    *it = Expression::StringLiteral(OxcBox::new_in(
      StringLiteral {
        span: assign.span,
        value: atom,
        raw: None,
        lone_surrogates: false,
      },
      self.allocator,
    ));
  }
}

fn get_item(expression: &Expression) -> Option<u32> {
  // Returns the starting span position for function/arrow/call expressions
  match expression {
    Expression::FunctionExpression(fn_expr) => {
      // Direct function expression
      return Some(fn_expr.span.start);
    }
    Expression::ArrowFunctionExpression(arrow_expr) => {
      // Arrow function expression
      return Some(arrow_expr.span.start);
    }
    Expression::CallExpression(call_expr) => {
      // Recursively search arguments for a function/arrow expression
      // Used to handle something like React.memo(() => { ... }) or React.forwardRef(function() { ... })
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
    // Other expression types are ignored
    _ => None,
  }
}

fn camel_case_to_kebab_case(input: &str) -> String {
  let mut result = String::new();
  let mut chars = input.chars().peekable();

  while let Some(ch) = chars.next() {
    if ch.is_uppercase() {
      if !result.is_empty() {
        result.push('-');
      }
      result.push(ch.to_lowercase().next().unwrap());
    } else {
      result.push(ch);
    }
  }

  result
}

/// Builds a CSS style string from an object expression (used for flair({...}) calls)
fn build_style_string_from_object(object_expression: &ObjectExpression) -> String {
  let mut style_string = String::new();

  for prop in &object_expression.properties {
    if let ObjectPropertyKind::ObjectProperty(object_property) = prop {
      // Extract the property key (string or identifier)
      let key = match &object_property.key {
        PropertyKey::StringLiteral(string_key) => Some(string_key.value.as_str()),
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
      };

      // Default separator and suffix for CSS property
      let mut separator = String::from(":");
      let mut suffix = String::from(";");

      if let Some(key) = key {
        // Extract the value for the property
        let value = match &object_property.value {
          Expression::StringLiteral(string_literal) => string_literal.value.to_string(),
          Expression::NumericLiteral(numeric_literal) => numeric_literal.value.to_string(),
          Expression::BooleanLiteral(boolean_literal) => {
            if boolean_literal.value {
              "true".to_string()
            } else {
              "false".to_string()
            }
          }
          // Nested object means selector (e.g., '&:hover': {...})
          Expression::ObjectExpression(nested_object) => {
            let object = nested_object.as_ref();
            separator = String::from(" ");
            suffix = String::from("");
            format!("{{ {} }}", build_style_string_from_object(object))
          }
          // Other types are ignored
          _ => "".to_string(),
        };

        // Convert camelCase to kebab-case only for CSS properties (not selectors)
        let css_property_name = match &object_property.value {
          Expression::ObjectExpression(_) => {
            // Selector: do not convert key
            key.to_string()
          }
          _ => {
            // CSS property: convert key
            camel_case_to_kebab_case(key)
          }
        };

        // Format as CSS property or selector
        style_string.push_str(&format!(
          "{}{separator} {}{suffix}\n",
          css_property_name, value
        ));
      }
    }
  }

  style_string
}
