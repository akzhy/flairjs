use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::flair_property::{FlairProperty, FLAIR_REPLACEMENT};
use crate::log_warn;
use crate::logger::{get_logger, LogEntry};
use crate::style_tag::StyleDetector;
use crate::update_attribute::ClassNameReplacer;
use crate::{log_error, parse_css::parse_css, update_attribute::SymbolStore};
use indexmap::IndexMap;
use lightningcss::css_modules::CssModuleExport;
use lightningcss::stylesheet::ToCssResult;
use napi::bindgen_prelude::Function as NapiFunction;
use napi::Env;
use napi_derive::napi;
use oxc::ast::ast::{
  ArrowFunctionExpression, BindingPatternKind, Class, FunctionBody, ImportDeclaration,
  ImportDeclarationSpecifier, ImportOrExportKind, ImportSpecifier, JSXChild, ModuleExportName,
  Statement, VariableDeclaration,
};
use oxc::ast::ast::{Expression, Program};
use oxc::ast::NONE;
use oxc::ast::{ast::Function, AstBuilder};
use oxc::codegen::{Codegen, CodegenOptions};
use oxc::span::SPAN;
use oxc::{
  allocator::Allocator,
  ast::ast::SourceType,
  ast_visit::{walk_mut, Visit, VisitMut},
  parser::{Parser, ParserReturn},
  semantic::{ScopeFlags, Scoping, SemanticBuilder, SymbolId},
};

/// Represents the different passes of the AST transformation.
/// The transformation requires three passes due to dependency chains:
///
/// **Pass 1 (CSS Extraction)**: Extract all CSS from flair properties and convert to CSS Modules format.
/// This must happen first because we need the CSS module class name mappings before we can replace anything.
///
/// **Pass 2 (Direct Replacements)**: Replace direct className references with CSS module hashed class names.
/// Example: `<div className="button" />` → `<div className="button_abc123" />`
/// However, this pass may encounter variable references like `<div className={myVar} />` where `myVar = "button"`.
/// These variables cannot be replaced yet because we need to trace their definitions first.
///
/// **Pass 3 (Variable Replacements)**: Replace variables that contain class names with their CSS module equivalents.
/// Example: `const className = "button"; <div className={className} />` → `const className = "button_abc123"; <div className={className} />`
/// This pass handles the variable definitions that were identified during Pass 2.
#[derive(PartialEq, Debug, Clone)]
enum Pass {
  First,
  Second,
  Third,
}

#[napi(object)]
pub struct Theme {
  pub breakpoints: HashMap<String, String>,
  pub prefix: Option<String>,
}

/// The import paths for flair-related utilities and components
const IMPORT_PATH: &str = "@flairjs/client";

#[napi(object)]
pub struct TransformOptions {
  pub css_out_dir: String,
  pub class_name_list: Option<Vec<String>>,
  pub use_theme: Option<bool>,
  pub theme: Option<Theme>,
  pub append_timestamp_to_css_file: Option<bool>,
}

#[napi(object)]
pub struct TransformOutput {
  pub code: String,
  pub sourcemap: Option<String>,
  pub css: String,
  pub logs: Vec<LogEntry>,
  pub generated_css_name: Option<String>,
}

/// Entry point for transforming a TypeScript React file.
/// This function performs CSS-in-JS transformation by:
/// 1. Parsing the TypeScript/JSX code into an AST
/// 2. Running a three-pass transformation to extract and process CSS
/// 3. Generating the transformed code with CSS imports
/// 4. Writing the extracted CSS to a separate file
pub fn transform(
  code: String,
  file_path: String,
  options: TransformOptions,
  css_preprocessor: Option<NapiFunction<String, String>>,
  env: Option<Env>,
) -> Option<TransformOutput> {
  if !matches!(
    file_path.split('.').next_back(),
    Some("tsx" | "jsx" | "ts" | "js")
  ) {
    return None;
  }

  // Set up the OXC parser infrastructure
  let allocator = Allocator::default();
  let source_type = match SourceType::from_path(&file_path) {
    Ok(source_type) => source_type,
    Err(_) => {
      log_error!(
        "Failed to determine source type from file path: {}",
        file_path
      );
      return None;
    }
  };

  let sourcemap_file_path = file_path.clone();

  // Parse the source code into an Abstract Syntax Tree (AST)
  let ParserReturn { mut program, .. } = Parser::new(&allocator, &code, source_type).parse();

  // Build semantic information (symbol tables, scopes, references)
  let semantic_builder = SemanticBuilder::new().build(&program);

  // Convert semantic info into scoping data for symbol resolution
  let scoping = semantic_builder.semantic.into_scoping();

  let ast_builder = AstBuilder::new(&allocator);

  // Create the main visitor that will perform the three-pass transformation
  let mut visitor = TransformVisitor::new(
    &allocator,
    ast_builder,
    &scoping,
    file_path.clone(),
    options,
    &css_preprocessor,
    env,
  );

  // Execute the multi-pass transformation on the AST
  visitor.begin(&mut program);

  // Generate the final JavaScript/TypeScript code with source maps
  let codegen = Codegen::new();
  let codegen = codegen.with_options(CodegenOptions {
    source_map_path: Some(PathBuf::from(&sourcemap_file_path)),
    ..CodegenOptions::default()
  });
  let result = codegen.build(&program);

  let result_code: String = result.code;

  // Convert source map to JSON string if available
  let sourcemap: Option<String> = result.map.map(|map| map.to_json_string());

  // Collect all logs that were accumulated during transformation
  let logs = get_logger().drain_logs();

  Some(TransformOutput {
    code: result_code,
    sourcemap,
    css: visitor.extracted_css.join("\n"),
    logs,
    generated_css_name: visitor.generated_css_name,
  })
}

/// Represents raw CSS data along with its scoping information.
#[derive(Clone, Debug)]
pub struct CSSData {
  pub raw_css: String,
  pub is_global: bool,
}

/// Main visitor struct that orchestrates the multi-pass CSS-in-JS transformation.
///
/// **Why Three Passes?**
/// The three-pass approach is necessary due to dependency chains in the transformation:
///
/// 1. **Pass 1**: Extract CSS and generate CSS module mappings (original → hashed class names)
/// 2. **Pass 2**: Replace direct className references, but identify variables needing replacement
/// 3. **Pass 3**: Replace variable declarations that contain class names
///
/// **Example transformation flow:**
/// ```tsx
/// // Original code:
/// const myClass = "button";
/// function Component() {
///   return <div className={myClass} />;
/// }
///
/// Component.flair = `.button { color: red; }`;
/// ```
///
/// Pass 1: Extract CSS, generate mapping: "button" → "button_abc123"
/// Pass 2: Can't replace myClass yet, but identifies it needs replacement
/// Pass 3: Replace variable: const myClass = "button_abc123";
///
struct TransformVisitor<'a> {
  allocator: &'a Allocator,
  options: TransformOptions,
  css_preprocessor: &'a Option<NapiFunction<'a, String, String>>,
  /// Symbols for imported "Style" components from flair packages
  style_tag_import_symbols: Vec<SymbolId>,
  /// Symbols for imported "c" / "cn" and other utility functions from flair packages  
  classname_util_symbols: Vec<SymbolId>,
  /// Accumulated CSS strings that will be written to the output CSS file
  extracted_css: Vec<String>,
  /// Maps function/component IDs to their raw CSS content before processing
  /// The id is actually the function's span.start position
  function_id_to_raw_css_mapping: IndexMap<u32, Vec<CSSData>>,
  /// Maps function/component IDs to their processed CSS module exports (class name mappings)
  css_module_exports: HashMap<u32, HashMap<String, CssModuleExport>>,

  ast_builder: AstBuilder<'a>,
  scoping: &'a Scoping,

  /// Tracks which identifiers requires CSS class name replacement
  identifier_symbol_ids: Vec<SymbolStore>,
  /// Current transformation pass
  pass: Pass,
  /// Variable symbol linking for resolving assignments. E.g.,
  /// ```
  /// const cl = "button";
  /// const cl2 = cl;
  ///
  /// return <div className={cl2} />;
  /// ```
  variable_linking: HashMap<SymbolId, SymbolId>,

  /// Visitor for detecting and processing flair properties in JSX/expressions
  flair_property_visitor: FlairProperty<'a>,

  /// Span start positions of style tags that should be removed from JSX
  style_tag_symbols: Vec<u32>,

  file_path: String,
  js_env: Option<Env>,

  /// Maps function span.start to its class span.start
  /// Used to handle method definitions inside classes
  /// e.g. class MyComponent { render() { return <div className="test" />; } }
  /// Here, the function_id_to_class_map will map the render() method's span.start to the MyComponent class's span.start
  /// This allows us to link className variables used inside methods to the correct function/component
  fn_id_to_class_map: HashMap<u32, u32>,

  parent_class_id: Option<u32>,

  generated_css_name: Option<String>,
}

impl<'a> TransformVisitor<'a> {
  fn new(
    allocator: &'a Allocator,
    ast_builder: AstBuilder<'a>,
    scoping: &'a Scoping,
    file_path: String,
    options: TransformOptions,
    css_preprocessor: &'a Option<NapiFunction<'a, String, String>>,
    js_env: Option<Env>,
  ) -> Self {
    let extracted_css = vec![];
    let identifier_symbol_ids: Vec<SymbolStore> = vec![];
    let css_module_exports: HashMap<u32, HashMap<String, CssModuleExport>> = HashMap::new();
    let style_tag_symbols: Vec<u32> = vec![];
    let style_tag_import_symbols: Vec<SymbolId> = vec![];
    let classname_util_symbols: Vec<SymbolId> = vec![];

    let variable_linking = HashMap::new();
    let flair_property_visitor = FlairProperty::new(scoping, allocator);

    Self {
      allocator,
      css_preprocessor,
      style_tag_import_symbols,
      style_tag_symbols,
      classname_util_symbols,
      extracted_css,
      variable_linking,
      ast_builder,
      scoping,
      identifier_symbol_ids,
      pass: Pass::First,
      css_module_exports,
      file_path,
      options,
      js_env,
      function_id_to_raw_css_mapping: IndexMap::new(),
      flair_property_visitor,
      fn_id_to_class_map: HashMap::new(),
      parent_class_id: None,
      generated_css_name: None,
    }
  }

  /// Orchestrates the three-pass transformation process:
  ///
  /// **Pass 1**: Extract CSS from flair properties and convert to CSS Modules format.
  /// This creates the mapping from original class names to hashed class names.
  ///
  /// **Pass 2**: Replace direct className references with CSS module hashed names.
  /// Also removes style tags from JSX and identifies variables that need replacement.
  ///
  /// **Pass 3**: Replace variable declarations that contain class names with their hashed equivalents.
  /// This handles cases like `const myClass = "button"` where the variable is used in JSX.
  fn begin(&mut self, program: &mut Program<'a>) {
    // Pass 1: Extract CSS and build CSS module mappings
    self.visit_program(program);

    // Process all collected CSS between passes to generate the class name mappings
    self.process_css();

    // Pass 2: Replace direct class name references and identify variables needing replacement
    self.pass = Pass::Second;
    self.visit_program(program);

    // Pass 3: Replace variable declarations that were identified in Pass 2
    self.pass = Pass::Third;
    self.visit_program(program);

    // Remove temporary flair statements from the AST
    Self::remove_flair_statements(program);

    // Generate a unique hash for this file's CSS output
    let hash = {
      let mut hasher = DefaultHasher::new();
      self.file_path.hash(&mut hasher);
      hasher.finish()
    };

    // Create the CSS file path and import statement
    let current_timestamp = if self.options.append_timestamp_to_css_file.unwrap_or(false) {
      let now = SystemTime::now();
      let duration_since_epoch = match now.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_millis(),
        Err(_) => {
          log_warn!("Failed to get duration since UNIX_EPOCH, using fallback timestamp");
          std::time::Duration::from_secs(0).as_millis()
        }
      };
      format!("-{}", duration_since_epoch)
    } else {
      "".to_string()
    };
    let hash_string = format!("{:x}{}.css", hash, current_timestamp);
    let import_path = format!("@flairjs/client/generated-css/{}", hash_string);

    // Create an import statement for the generated CSS file
    let import_statement = Statement::from(
      self.ast_builder.module_declaration_import_declaration(
        SPAN,
        None, // No specifiers for side-effect-only import
        self
          .ast_builder
          .string_literal(SPAN, self.allocator.alloc_str(&import_path), None),
        None,
        NONE,
        ImportOrExportKind::Value,
      ),
    );

    // Write the extracted CSS to a file in the specified output directory
    let file_path = format!("{}/{}", self.options.css_out_dir, hash_string);
    match File::create(&file_path) {
      Ok(mut file) => {
        if let Err(err) = file.write_all(self.extracted_css.join("\n").as_bytes()) {
          log_error!(
            "Failed to write CSS to file: {}, reason: {:#?}",
            file_path,
            err
          );
        }
      }
      Err(err) => {
        log_error!(
          "Failed to create file in css_out_dir: {}, reason: {:#?}",
          self.options.css_out_dir,
          err
        );
      }
    }

    self.generated_css_name = Some(hash_string);
    // Insert the CSS import at the top of the transformed file
    program.body.insert(0, import_statement);
  }

  /// Remove __flair_replacement__ statements from the AST
  fn remove_flair_statements(program: &mut Program<'a>) {
    program.body.retain(|stmt| {
      if let Statement::ExpressionStatement(expr_stmt) = stmt {
        if let Expression::StringLiteral(string_lit) = &expr_stmt.expression {
          if string_lit.value == FLAIR_REPLACEMENT {
            return false;
          }
        }
      }
      true
    });
  }

  /// Process the collected CSS data, apply preprocessing and parsing, and store the results.
  /// This method handles both scoped and global CSS:
  /// - Scoped CSS: Processed with CSS modules to generate unique class names
  /// - Global CSS: Processed as-is without scoping
  ///
  /// Both types can be preprocessed using a custom CSS preprocessor function
  fn process_css(&mut self) {
    let flair_scoped_styles = self.flair_property_visitor.get_scoped_style();
    let flair_global_styles = self.flair_property_visitor.get_global_style();

    // Consolidate all CSS styles by function/component ID
    for (span_start, style) in flair_scoped_styles.iter().chain(flair_global_styles.iter()) {
      self
        .function_id_to_raw_css_mapping
        .entry(*span_start)
        .or_default()
        .push(style.to_owned());
    }

    // Process each function's CSS styles
    self
      .function_id_to_raw_css_mapping
      .iter()
      .enumerate()
      .for_each(|(index, (fn_id, styles))| {
        // Separate scoped and global CSS for different processing
        let (scoped_css, global_css) = {
          let mut scoped_css: Option<String> = None;
          let mut global_css: Option<String> = None;
          styles.iter().for_each(|style| {
            if style.is_global {
              global_css
                .get_or_insert_with(String::new)
                .push_str(&style.raw_css);
            } else {
              scoped_css
                .get_or_insert_with(String::new)
                .push_str(&style.raw_css);
            }
          });

          (scoped_css, global_css)
        };

        // Apply CSS preprocessing if available.
        let (preprocessed_scoped_css, preprocessed_global_css) = {
          if self.js_env.is_some() {
            if let Some(preprocessor) = &self.css_preprocessor {
              // Apply preprocessor to scoped CSS, fallback to original on error
              let scoped_result = scoped_css.as_ref().map(|original| {
                preprocessor
                  .call(original.clone())
                  .unwrap_or(original.clone())
              });

              // Apply preprocessor to global CSS, fallback to original on error
              let global_result = global_css.as_ref().map(|original| {
                preprocessor
                  .call(original.clone())
                  .unwrap_or(original.clone())
              });

              (scoped_result, global_result)
            } else {
              // No preprocessor available, use original CSS
              (scoped_css, global_css)
            }
          } else {
            // No JavaScript environment, use original CSS
            (scoped_css, global_css)
          }
        };

        let use_theme = self.options.use_theme.unwrap_or(false);

        // Parse scoped CSS with CSS modules enabled for class name generation
        let parsed_scoped_css: Option<ToCssResult> =
          preprocessed_scoped_css.clone().and_then(|css| {
            let res = parse_css(
              &css,
              &format!("{}:{}", self.file_path, index),
              true, // Enable CSS modules for scoped styles
              use_theme,
              &self.options.theme,
            );

            match res {
              Ok(val) => Some(val),
              Err(_) => {
                log_error!(
                  "Failed to parse CSS in function starting at {}: {:#?}. CSS: {:#?}",
                  fn_id,
                  res.err(),
                  preprocessed_scoped_css
                );
                None
              }
            }
          });

        // Parse global CSS without CSS modules
        let parsed_global_css: Option<ToCssResult> =
          preprocessed_global_css.clone().and_then(|css| {
            let res = parse_css(
              &css,
              &format!("{}:{}", self.file_path, fn_id),
              false, // Disable CSS modules for global styles
              use_theme,
              &self.options.theme,
            );

            match res {
              Ok(val) => Some(val),
              Err(_) => {
                log_error!(
                  "Failed to parse CSS in function starting at {}: {:#?}. CSS: {:#?}",
                  fn_id,
                  res.err(),
                  preprocessed_global_css
                );
                None
              }
            }
          });

        // Store CSS module exports for class name replacement in Pass 2
        if let Some(parsed_scoped_css) = parsed_scoped_css {
          let empty_exports = HashMap::new();
          let css_exports = parsed_scoped_css.exports.as_ref().unwrap_or(&empty_exports);

          self.css_module_exports.insert(*fn_id, css_exports.clone());

          self.extracted_css.push(parsed_scoped_css.code);
        }

        // Add global CSS directly to the output (no class name mapping needed)
        if let Some(parsed_global_css) = parsed_global_css {
          self.extracted_css.push(parsed_global_css.code);
        }
      });
  }

  /// Processes function bodies differently based on the current transformation pass.
  /// This method coordinates the different phases of transformation for each function.
  fn process_function_body(&mut self, body: &mut FunctionBody<'a>, fn_start: u32) {
    match self.pass {
      Pass::First => {
        // Detect and collect style tag information and CSS content
        let mut style_detector = StyleDetector::new(self.scoping, &self.style_tag_import_symbols);
        style_detector.visit_function_body(body);

        if let Some(class_id) = self.parent_class_id {
          self.fn_id_to_class_map.insert(fn_start, class_id);
        }

        // If style tags were detected, store the CSS and symbol information
        if style_detector.has_style {
          self.style_tag_symbols = style_detector.get_style_tag_symbol_ids();

          // If this function is inside a class, map it to the parent class for CSS scope inheritance
          if let Some(class_id) = self.parent_class_id {
            self
              .function_id_to_raw_css_mapping
              .insert(class_id, style_detector.css);
          } else {
            self
              .function_id_to_raw_css_mapping
              .insert(fn_start, style_detector.css);
          }
        }
      }
      Pass::Second => {
        // Replace direct className references and identify variables that need replacement in Pass 3
        let mut classname_replacer = ClassNameReplacer {
          allocator: self.allocator,
          class_name_map: self.get_css_exports(&fn_start).unwrap_or_default(),
          ast_builder: self.ast_builder,
          scoping: self.scoping,
          identifier_symbol_ids: self.identifier_symbol_ids.clone(),
          fn_id: fn_start,
          classname_util_symbols: self.classname_util_symbols.clone(),
          variable_linking: self.variable_linking.clone(),
          class_name_list: self
            .options
            .class_name_list
            .clone()
            .unwrap_or(vec!["className".to_string(), "class".to_string()]),
        };

        classname_replacer.visit_function_body(body);

        // Update our tracking of which identifiers need to be processed in Pass 3
        self.identifier_symbol_ids = classname_replacer.get_identifier_symbol_ids().to_vec();
      }
      Pass::Third => {
        // Pass 3 is handled at the variable declaration level, not at the function body level
        // This is because variable replacements happen globally, not per function
      }
    }
  }

  /// Retrieves CSS module exports for a given function ID.
  /// For class methods, this looks up the parent class's CSS exports instead
  /// of the method's own exports.
  fn get_css_exports(&self, fn_id: &u32) -> Option<HashMap<String, CssModuleExport>> {
    // Check if this function is a method inside a class
    if let Some(class_id) = self.fn_id_to_class_map.get(fn_id) {
      // Use the parent class's CSS exports for consistency
      self.css_module_exports.get(class_id).cloned()
    } else {
      // Use the function's own CSS exports
      self.css_module_exports.get(fn_id).cloned()
    }
  }

  fn get_import_symbol(&self, import_specifier: &ImportSpecifier, name: &str) -> Option<SymbolId> {
    if let ModuleExportName::IdentifierName(identifier) = &import_specifier.imported {
      if identifier.name == name {
        return Some(import_specifier.local.symbol_id());
      }
    }

    if import_specifier.local.name == name {
      return Some(import_specifier.local.symbol_id());
    }

    None
  }
}

impl<'a> VisitMut<'a> for TransformVisitor<'a> {
  /// Processes import declarations to identify flair-related imports.
  /// Tracks symbols for "Style" components and "c" utility functions from flair packages.
  fn visit_import_declaration(&mut self, it: &mut ImportDeclaration<'a>) {
    if it.source.value.as_str().starts_with(IMPORT_PATH) && self.pass == Pass::First {
      let specifiers = it.specifiers.as_ref();
      if let Some(specifiers) = specifiers {
        specifiers.iter().for_each(|specifier| {
          if let ImportDeclarationSpecifier::ImportSpecifier(import_specifier) = specifier {
            // Track the "Style" component import for style tag detection
            let style_import_symbol = self.get_import_symbol(import_specifier, "Style");
            if let Some(symbol_id) = style_import_symbol {
              self.style_tag_import_symbols.push(symbol_id);
            }
            // Track the "c" utility function import for className processing
            let c_import_symbol = self.get_import_symbol(import_specifier, "c");
            let cn_import_symbol = self.get_import_symbol(import_specifier, "cn");
            if let Some(symbol_id) = c_import_symbol {
              self.classname_util_symbols.push(symbol_id);
            }
            if let Some(symbol_id) = cn_import_symbol {
              self.classname_util_symbols.push(symbol_id);
            }
          }
        });
      }
    }
  }

  /// Handles variable declarations across different transformation passes.
  /// Pass 1: Builds variable linking for symbol resolution
  /// Pass 2: No processing needed
  /// Pass 3: Processes variables that reference transformed CSS class names
  fn visit_variable_declaration(&mut self, it: &mut VariableDeclaration<'a>) {
    match self.pass {
      Pass::First => {
        // Build variable linking to track symbol assignments for complex variable chains
        // Example: const a = "button"; const b = a; <div className={b} />
        // This creates the chain: b -> a -> "button" so we can trace back to the original class name
        it.declarations.iter_mut().for_each(|decl| {
          if let BindingPatternKind::BindingIdentifier(binding_identifier) = &decl.id.kind {
            if let Some(Expression::Identifier(identifier)) = &decl.init {
              let symbol_id = binding_identifier.symbol_id();
              let reference = self.scoping.get_reference(identifier.reference_id());
              let init_symbol_id = reference.symbol_id();

              // Link the new variable symbol to the symbol it references
              // This allows us to trace variable assignments: const newVar = existingVar
              if let Some(init_symbol_id) = init_symbol_id {
                self.variable_linking.insert(symbol_id, init_symbol_id);
              }
            }
          }
        });

        // Also process flair properties in this pass
        self.flair_property_visitor.visit_variable_declaration(it);
      }
      Pass::Second => {
        // No special processing needed in second pass for variable declarations
      }
      Pass::Third => {
        // Replace variable declarations that contain class names with their CSS module equivalents
        // Example: const className = "button" -> const className = "button_abc123"
        it.declarations.iter_mut().for_each(|decl| {
          if let BindingPatternKind::BindingIdentifier(binding_identifier) = &decl.id.kind {
            let symbol_id = binding_identifier.symbol_id();

            // Check if this variable was identified in Pass 2 as needing CSS class name replacement
            let symbol_store_item = self
              .identifier_symbol_ids
              .iter()
              .find(|s| s.symbol_id == symbol_id);

            if let Some(symbol_store_item) = symbol_store_item {
              // Get the CSS exports for the function context where this variable was used
              let css_exports = self.get_css_exports(&symbol_store_item.fn_id);
              if css_exports.is_none() {
                return;
              }

              // Apply class name replacement to the variable's initialization expression
              // This transforms: const myClass = "button" -> const myClass = "button_abc123"
              let mut classname_replacer = ClassNameReplacer {
                allocator: self.allocator,
                class_name_map: css_exports.unwrap_or_default(),
                ast_builder: self.ast_builder,
                scoping: self.scoping,
                identifier_symbol_ids: vec![],
                fn_id: decl.span.start,
                classname_util_symbols: self.classname_util_symbols.clone(),
                variable_linking: self.variable_linking.clone(),
                class_name_list: self
                  .options
                  .class_name_list
                  .clone()
                  .unwrap_or(vec!["className".to_string(), "class".to_string()]),
              };

              if decl.init.is_some() {
                classname_replacer.update_expression(decl.init.as_mut());
              }
            }
          }
        });
      }
    }

    walk_mut::walk_variable_declaration(self, it);
  }

  /// Processes expressions, handling flair properties in Pass 1 and removing style tags in Pass 2.
  fn visit_expression(&mut self, it: &mut Expression<'a>) {
    if self.pass == Pass::First {
      // Collect flair property information during the first pass
      self.flair_property_visitor.visit_expression(it);
    } else if self.pass == Pass::Second {
      // Remove Style components from JSX during the second pass
      if let Expression::JSXElement(jsx) = it {
        jsx.children.retain_mut(|child| {
          if let JSXChild::Element(element) = child {
            if self.style_tag_symbols.contains(&element.span.start) {
              return false;
            }
            return true;
          }
          true
        });
      } else if let Expression::JSXFragment(jsx) = it {
        jsx.children.retain_mut(|child| {
          if let JSXChild::Element(element) = child {
            // Remove elements that match detected Style component symbols
            if self.style_tag_symbols.contains(&element.span.start) {
              return false;
            }
            return true;
          }
          true
        });
      }
    }
    walk_mut::walk_expression(self, it);
  }

  fn visit_arrow_function_expression(&mut self, it: &mut ArrowFunctionExpression<'a>) {
    let body = it.body.as_mut();
    self.process_function_body(body, it.span.start);

    walk_mut::walk_arrow_function_expression(self, it);
  }

  fn visit_function(&mut self, function: &mut Function<'a>, flags: ScopeFlags) {
    let body = match function.body.as_mut() {
      Some(body) => body,
      None => {
        walk_mut::walk_function(self, function, flags);
        return;
      }
    };
    self.process_function_body(body, function.span.start);
    self.flair_property_visitor.visit_function(function);

    walk_mut::walk_function(self, function, flags);
  }

  /// Processes class declarations, setting up parent-child relationships for CSS scoping.
  /// The generated CSS is scoped to the full class and all methods will share the same CSS scope.
  fn visit_class(&mut self, it: &mut Class<'a>) {
    if self.pass == Pass::First {
      // Process flair properties in the class
      self.flair_property_visitor.visit_class(it);

      // Set the current class as parent context for any methods within it
      // This allows methods to inherit CSS scope from their containing class
      self.parent_class_id = Some(it.span.start);
      walk_mut::walk_class(self, it);

      // Clear the parent context when exiting the class
      self.parent_class_id = None;
    } else {
      // In other passes, just walk the class normally
      walk_mut::walk_class(self, it);
    }
  }
}
