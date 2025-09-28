use std::collections::HashMap;

use lightningcss::css_modules::CssModuleExport;
use oxc::allocator::Allocator;
use oxc::allocator::Box as OxcBox;
use oxc::ast::ast::StaticMemberExpression;
use oxc::ast::ast::TemplateLiteral;
use oxc::ast::{
  ast::{
    ArrayExpression, BinaryExpression, BinaryOperator, CallExpression, ConditionalExpression,
    Expression, IdentifierReference, JSXAttributeName, JSXAttributeValue, JSXExpression,
    LogicalExpression, LogicalOperator, ObjectExpression, ObjectPropertyKind, PropertyKey,
    StringLiteral,
  },
  AstBuilder,
};
use oxc::{
  ast::ast::JSXAttribute,
  ast_visit::{walk_mut, VisitMut},
  semantic::{Scoping, SymbolId},
};
use regex::Regex;

/// Stores a symbol ID along with its associated function ID for tracking
/// variable references that need to be processed in a later pass
/// During the second pass of transformation, when we encounter variables that
/// reference class names, we can't update them directly, so we store them here
#[derive(Clone, Debug, PartialEq)]
pub struct SymbolStore {
  pub symbol_id: SymbolId,
  /// The span.start of the function containing this variable reference
  /// Used to scope class name variables to their containing function
  ///
  /// Currently, after lightningcss processing, the output css is mapped to its parent function.
  /// So, the hashed classnames and styles are available in the hashmap HashMap<fn_id, Data>.
  ///
  /// During the third pass, which updates variable references, we need to know which function
  /// the variable belongs to, so we can look up the correct hashed classnames from the hashmap.
  /// This fn_id helps us scope variable references to their containing function.
  pub fn_id: u32,
}

impl SymbolStore {
  pub fn new(symbol_id: SymbolId, fn_id: u32) -> Self {
    Self { symbol_id, fn_id }
  }
}

/// Main struct responsible for transforming CSS class names in JSX/TSX files
/// This handles CSS modules transformation by replacing original class names
/// with their hashed/scoped equivalents
pub struct ClassNameReplacer<'a> {
  /// Maps original class names to their CSS module exports (hashed names)
  pub class_name_map: HashMap<String, CssModuleExport>,
  pub allocator: &'a Allocator,
  pub ast_builder: AstBuilder<'a>,
  pub scoping: &'a Scoping,
  /// Tracks all identifier symbols found during transformation that reference variables
  /// These can't be updated directly during second pass, so they're stored for later processing
  pub identifier_symbol_ids: Vec<SymbolStore>,
  /// Current function ID for scope tracking (span.start of the current function)
  /// Class names are scoped to functions.
  pub fn_id: u32,
  /// Symbol IDs of utility functions that handle class names (e.g., c, cn) imported from client library
  pub classname_util_symbols: Vec<SymbolId>,
  /// Maps variable symbols to their linked/aliased symbols for resolution
  /// Currently handles simple cases like: const a = b; b = "className"
  /// Future enhancement: support all kinds of variable assignments including
  /// destructuring, imports, and complex assignment patterns
  pub variable_linking: HashMap<SymbolId, SymbolId>,
  /// List of attribute names to process (e.g., ["className", "class"])
  /// Supports regex patterns wrapped in forward slashes
  pub class_name_list: Vec<String>,
}

impl<'a> ClassNameReplacer<'a> {
  fn get_classname_exports(&self) -> &HashMap<String, CssModuleExport> {
    &self.class_name_map
  }

  pub fn get_identifier_symbol_ids(&self) -> &Vec<SymbolStore> {
    &self.identifier_symbol_ids
  }

  /// Checks if a given attribute name should be processed for class name transformation
  /// Supports both exact string matching and regex patterns (wrapped in forward slashes)
  /// Example: "className" matches exactly, "/class.*/" matches "className", "class", etc.
  fn is_classname_in_list(&self, class_name: &str) -> bool {
    let item = self.class_name_list.iter().find(|item| {
      // Check if the item is a regex pattern
      if item.starts_with("/") && item.ends_with("/") {
        // Extract the regex pattern by removing the surrounding slashes
        let pattern = &item[1..item.len() - 1];
        if let Ok(re) = Regex::new(pattern) {
          return re.is_match(class_name);
        }
      }
      // Fallback to exact string matching
      *item == class_name
    });

    item.is_some()
  }

  /// Transforms class names from original to CSS module equivalents
  /// Handles space-separated class names (e.g., "btn primary" -> "btn_abc123 primary_def456")
  /// If a class name isn't found in the CSS module map, it's left unchanged
  fn get_updated_classname(&self, class_name: &str) -> String {
    let class_names: Vec<&str> = class_name.split(' ').collect();
    let mut updated_class_names = Vec::new();

    for class_name in class_names {
      if let Some(export) = self.get_classname_exports().get(class_name) {
        updated_class_names.push(export.name.clone());
      } else {
        updated_class_names.push(class_name.to_string());
      }
    }

    updated_class_names.join(" ")
  }

  /// Updates string literals containing class names
  /// Transforms the class names and creates a new atom in the allocator
  fn update_string_expression(&mut self, string_value: &mut OxcBox<'a, StringLiteral<'a>>) {
    let updated_class_names_str = self.get_updated_classname(&string_value.value);
    // Create a new atom in the allocator for the updated string
    let atom = self
      .ast_builder
      .atom(self.allocator.alloc_str(&updated_class_names_str));

    string_value.value = atom;
  }

  /// Updates array expressions that may contain class names
  /// Recursively processes each element in the array
  /// Example: ["btn", "primary"] -> each string element gets transformed
  fn update_array_expression(&mut self, array_expression: &mut OxcBox<'a, ArrayExpression<'a>>) {
    array_expression.elements.iter_mut().for_each(|element| {
      self.update_expression(element.as_expression_mut());
    });
  }

  /// Updates object expressions where keys might be class names
  /// Only processes string literal keys, not computed or identifier keys
  /// Example: { "btn": true, "primary": false } -> transforms the string keys
  fn update_object_expression(&mut self, object_expression: &mut OxcBox<'a, ObjectExpression<'a>>) {
    for prop in &mut object_expression.properties {
      if let ObjectPropertyKind::ObjectProperty(property) = prop {
        // Only process string literal keys, not computed properties or identifiers
        if let PropertyKey::StringLiteral(string_key) = &mut property.key {
          self.update_string_expression(string_key);
        }
      }
    }
  }

  /// Processes arguments for any call expression
  /// Also handles array.join(" ") patterns
  /// This handles common patterns like: ["btn", "primary"].join(" "), clsx("btn", { "active": isActive })
  fn update_call_expression(&mut self, call_expression: &mut OxcBox<'a, CallExpression<'a>>) {
    // Destructure the call expression to access callee and arguments
    let CallExpression {
      callee, arguments, ..
    } = call_expression.as_mut();

    // Special case: Handle array.join(" ") which is a common pattern for class names
    if let Expression::StaticMemberExpression(static_member) = callee {
      let StaticMemberExpression {
        property, object, ..
      } = static_member.as_mut();

      // Check if we're calling join on an array
      if let Expression::ArrayExpression(array_expression) = object {
        // Verify this is specifically join(" ") - joining with a space
        let is_string_join = arguments.first().is_some_and(|arg| {
          if let Some(Expression::StringLiteral(string_lit)) = arg.as_expression() {
            return string_lit.value.as_str() == " ";
          }
          false
        });

        // If this is array.join(" "), process the array elements and return early
        if property.name == "join" && is_string_join {
          array_expression.elements.iter_mut().for_each(|element| {
            self.update_expression(element.as_expression_mut());
          });
          return;
        }
      }
    }

    // For all other call expressions, process the arguments
    call_expression.arguments.iter_mut().for_each(|arg| {
      self.update_expression(arg.as_expression_mut());
    });
  }

  /// Updates conditional expressions (ternary operators)
  /// Processes both the consequent (true branch) and alternate (false branch)
  /// Example: condition ? "btn" : "btn-disabled"
  fn update_conditional_expression(
    &mut self,
    conditional_expression: &mut OxcBox<'a, ConditionalExpression<'a>>,
  ) {
    self.update_expression(Some(&mut conditional_expression.consequent));
    self.update_expression(Some(&mut conditional_expression.alternate));
  }

  /// Updates logical expressions (||, && and ??)
  /// For OR (||) and nullish coalescing (??): both operands are processed
  /// For AND (&&): only the right operand is processed since that's the value
  /// that will be used when the condition is true
  /// Examples:
  /// - `className || "default"` - both sides processed
  /// - `isActive && "active"` - only "active" processed
  fn update_logical_expression(
    &mut self,
    logical_expression: &mut OxcBox<'a, LogicalExpression<'a>>,
  ) {
    // For OR (||) and nullish coalescing (??), process the left operand
    if logical_expression.operator == LogicalOperator::Or
      || logical_expression.operator == LogicalOperator::Coalesce
    {
      self.update_expression(Some(&mut logical_expression.left));
    }
    // Always process the right operand for all logical operators
    // For AND (&&), only the right side matters? as it's the actual value used
    self.update_expression(Some(&mut logical_expression.right));
  }

  /// Updates binary expressions, specifically string concatenation with +
  /// Only processes Addition operations as they might concatenate class names
  /// Example: "btn " + "primary" -> both sides need to be transformed
  fn update_binary_expression(&mut self, binary_expression: &mut OxcBox<'a, BinaryExpression<'a>>) {
    // Only process addition operations (string concatenation for class names)
    if BinaryOperator::Addition == binary_expression.operator {
      self.update_expression(Some(&mut binary_expression.left));
      self.update_expression(Some(&mut binary_expression.right));
    }
    // Other binary operators (-, *, /, etc.) are not processed as they're unlikely
    // to be used for class name construction
  }

  /// Recursively resolves variable linking to get the final symbol ID
  /// Currently handles simple variable assignments like: const a = b; b = "className"
  /// Future enhancement: support complex assignment patterns.
  fn get_resolved_symbol_id(&self, symbol_id: SymbolId) -> SymbolId {
    // If this symbol is linked to another symbol, recursively resolve it
    // Eg: const b = "className"; const a = b;
    // <div className={a} />
    if let Some(&linked_symbol_id) = self.variable_linking.get(&symbol_id) {
      // Recursive call to handle chains of variable linking (a -> b -> c -> "className")
      return self.get_resolved_symbol_id(linked_symbol_id);
    }
    // Return the original symbol ID if no linking is found
    // Eg: const a = "className"; <div className={a} />
    symbol_id
  }

  /// Updates identifier expressions by tracking their symbol IDs for later processing
  /// During the second pass of transformation, we can't directly update variable values
  /// that reference class names, so we collect them here to be processed in the third pass
  fn update_identifier_expression(&mut self, identifier_expression: &mut IdentifierReference<'a>) {
    // Get the semantic reference for this identifier
    let reference = self
      .scoping
      .get_reference(identifier_expression.reference_id());
    let symbol_id = reference.symbol_id();

    if let Some(symbol_id) = symbol_id {
      // Resolve any variable linking to get the final symbol
      let resolved_symbol_id = self.get_resolved_symbol_id(symbol_id);
      // Store the symbol ID along with the current function ID for later processing
      self
        .identifier_symbol_ids
        .push(SymbolStore::new(resolved_symbol_id, self.fn_id));
    }
  }

  /// Updates template literals (backtick strings) containing class names
  /// Processes both the static parts (quasis) and dynamic expressions
  /// Example: `btn ${isActive ? 'active' : ''} primary`
  fn update_template_expression(
    &mut self,
    template_expression: &mut OxcBox<'a, TemplateLiteral<'a>>,
  ) {
    // Update the static string parts of the template literal
    template_expression.quasis.iter_mut().for_each(|elem| {
      let updated_class_names_str = self.get_updated_classname(&elem.value.raw);
      let atom = self
        .ast_builder
        .atom(self.allocator.alloc_str(&updated_class_names_str));
      elem.value.raw = atom;
    });

    // Update the dynamic expressions within ${...}
    template_expression.expressions.iter_mut().for_each(|expr| {
      self.update_expression(Some(expr));
    });
  }

  /// Main entry point for updating any expression that might contain class names
  /// Dispatches to specific update methods based on the expression type
  pub fn update_expression(&mut self, expression: Option<&mut Expression<'a>>) {
    let Some(expression) = expression else {
      return;
    };
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
      Expression::TemplateLiteral(template_expression) => {
        self.update_template_expression(template_expression);
      }
      _ => {
        // Future enhancement: handle more complex expressions if needed
        // Utility functions like c, cn, etc. can handle complex expressions
      }
    }
  }
}

impl<'a> VisitMut<'a> for ClassNameReplacer<'a> {
  /// Visits call expressions to handle class name utility functions
  /// Special handling for functions like c, cn, etc.
  fn visit_call_expression(&mut self, it: &mut CallExpression<'a>) {
    // Check if the callee is an identifier (function name)
    if let Expression::Identifier(identifier_calle) = &it.callee {
      // Resolve the function's symbol to check if it's a class name utility
      let callee_ref = self.scoping.get_reference(identifier_calle.reference_id());
      let callee_symbol_id = callee_ref.symbol_id();

      if let Some(callee_symbol_id) = callee_symbol_id {
        // If this function is registered as a class name utility (like c, cn)
        if self.classname_util_symbols.contains(&callee_symbol_id) {
          // Process all arguments to the utility function
          it.arguments.iter_mut().for_each(|arg| {
            self.update_expression(arg.as_expression_mut());
          });
        }
      }
    }
    walk_mut::walk_call_expression(self, it);
  }

  /// Visits JSX attributes to find and transform class name attributes
  /// This is the main entry point for transforming className, class, etc.
  fn visit_jsx_attribute(&mut self, it: &mut JSXAttribute<'a>) {
    // Check if this is an attribute we care about (className, class, etc.)
    if let JSXAttributeName::Identifier(ident) = &it.name {
      if self.is_classname_in_list(&ident.name) {
        let Some(value) = it.value.as_mut() else {
          walk_mut::walk_jsx_attribute(self, it);
          return;
        };

        // Handle different types of attribute values
        if let JSXAttributeValue::StringLiteral(string_value) = value {
          // Simple string: className="btn primary"
          self.update_string_expression(string_value);
        } else if let JSXAttributeValue::ExpressionContainer(expr_container) = value {
          // Expression: className={someExpression}
          let expression = &mut expr_container.expression;

          // Handle different expression types within the JSX expression container
          match expression {
            JSXExpression::ArrayExpression(array_expression) => {
              self.update_array_expression(array_expression);
            }
            JSXExpression::CallExpression(call_expression) => {
              self.update_call_expression(call_expression);
            }
            JSXExpression::LogicalExpression(logical_expression) => {
              self.update_logical_expression(logical_expression);
            }
            JSXExpression::ConditionalExpression(conditional_expression) => {
              self.update_conditional_expression(conditional_expression);
            }
            JSXExpression::ObjectExpression(object_expression) => {
              self.update_object_expression(object_expression);
            }
            JSXExpression::BinaryExpression(binary_expression) => {
              self.update_binary_expression(binary_expression);
            }
            JSXExpression::Identifier(identifier_expression) => {
              self.update_identifier_expression(identifier_expression);
            }
            JSXExpression::TemplateLiteral(template_expression) => {
              self.update_template_expression(template_expression);
            }
            JSXExpression::StringLiteral(string_value) => {
              self.update_string_expression(string_value);
            }
            _ => {
              // Future enhancement: handle more complex expressions if needed
              // Utility functions like c, cn, etc. can handle complex expressions
            }
          }
        }
      }
    }
    walk_mut::walk_jsx_attribute(self, it);
  }
}
