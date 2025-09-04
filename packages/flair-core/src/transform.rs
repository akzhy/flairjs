use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;

use crate::update_attribute::ClassNameReplacer;
use crate::{parse_css::parse_css, update_attribute::SymbolStore};
use lightningcss::css_modules::CssModuleExport;
use napi::Env;
use napi::JsFunction;
use napi_derive::napi;
use oxc::{
  allocator::Allocator,
  ast::ast::SourceType,
  ast_visit::{walk_mut, Visit, VisitMut},
  parser::{Parser, ParserReturn},
  semantic::{ScopeFlags, Scoping, SemanticBuilder, SymbolId},
};
use oxc_ast::ast::{
  AssignmentTarget, BindingPatternKind, Expression, FunctionBody, ImportDeclarationSpecifier,
  ImportOrExportKind, JSXChild, JSXElementName, JSXExpression, Statement,
};
use oxc_ast::ast::{JSXElement, Program};
use oxc_ast::{ast::Function, AstBuilder};
use oxc_ast::{AstType, NONE};
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_span::SPAN;

#[derive(PartialEq, Debug, Clone)]
enum Pass {
  First,
  Second,
}

const IMPORT_PATH: &str = "jsx-styled-react";

#[napi(object)]
pub struct TransformOptions {
  pub code: String,
  pub file_path: String,
  pub css_out_dir: String,
}

#[napi(object)]
pub struct TransformOutput {
  pub code: String,
  pub sourcemap: Option<String>,
}

pub fn transform(
  options: TransformOptions,
  css_preprocessor: Option<JsFunction>,
  env: Option<Env>,
) -> Option<TransformOutput> {
  if !options.file_path.ends_with(".tsx") {
    return None;
  }

  let allocator = Allocator::default();
  let source_type = SourceType::from_path(&options.file_path).unwrap();

  let sourcemap_file_path = options.file_path.clone().replace(".tsx", ".tsx.map");

  let ParserReturn { mut program, .. } =
    Parser::new(&allocator, &options.code, source_type).parse();

  let semantic_builder = SemanticBuilder::new().build(&program);

  let scoping = semantic_builder.semantic.into_scoping();

  // let mut local_style_tag_name = "style".to_string();
  // let mut local_class_name_util_functions = vec![];
  let ast_builder = AstBuilder::new(&allocator);

  // Traverse the AST
  let mut visitor = TransformVisitor::new(
    &allocator,
    &css_preprocessor,
    ast_builder,
    &scoping,
    options.file_path.clone(),
    options.css_out_dir.clone(),
    env,
  );

  visitor.begin(&mut program);

  let codegen = Codegen::new();
  let codegen = codegen.with_options(CodegenOptions {
    source_map_path: Some(PathBuf::from(&sourcemap_file_path)),
    ..CodegenOptions::default()
  });
  let result = codegen.build(&program);

  let result_code: String = result.code;
  // println!("Transformedd result:\n{}", result_code);
  let sourcemap: Option<String> = {
    if result.map.is_some() {
      Some(result.map.unwrap().to_json_string())
    } else {
      None
    }
  };

  Some(TransformOutput {
    code: result_code,
    sourcemap: sourcemap,
  })
}

struct TransformVisitor<'a> {
  allocator: &'a Allocator,
  // local_style_tag_name: &'a mut String,
  style_tag_symbols: Vec<SymbolId>,
  classname_util_symbols: Vec<SymbolId>,
  // local_class_name_util_functions: &'a mut Vec<String>,
  extracted_css: Vec<String>,
  css_preprocessor: &'a Option<JsFunction>,
  ast_builder: AstBuilder<'a>,
  scoping: &'a Scoping,
  identifier_symbol_ids: Vec<SymbolStore>,
  pass: Pass,
  css_module_exports: HashMap<u32, HashMap<String, CssModuleExport>>,
  file_path: String,
  css_out_dir: String,
  js_env: Option<Env>,
  function_to_style_mapping: HashMap<SymbolId, Vec<String>>,
  function_names: Vec<(String, SymbolId)>,
  functions_stack: Vec<SymbolId>,
}

impl<'a> TransformVisitor<'a> {
  fn new(
    allocator: &'a Allocator,
    css_preprocessor: &'a Option<JsFunction>,
    ast_builder: AstBuilder<'a>,
    scoping: &'a Scoping,
    file_path: String,
    css_out_dir: String,
    js_env: Option<Env>,
  ) -> Self {
    let extracted_css = vec![];
    let symbold_ids_vec: Vec<SymbolStore> = vec![];
    let css_module_exports: HashMap<u32, HashMap<String, CssModuleExport>> = HashMap::new();
    let style_tag_symbols: Vec<SymbolId> = vec![];
    let classname_util_symbols: Vec<SymbolId> = vec![];

    let function_to_style_mapping = HashMap::new();
    let function_names = vec![];

    let functions_stack = vec![];

    Self {
      allocator,
      style_tag_symbols,
      classname_util_symbols,
      extracted_css,
      css_preprocessor,
      ast_builder,
      scoping,
      identifier_symbol_ids: symbold_ids_vec,
      pass: Pass::First,
      css_module_exports,
      file_path,
      css_out_dir,
      js_env,
      function_names,
      function_to_style_mapping,
      functions_stack,
    }
  }

  fn begin(&mut self, program: &mut Program<'a>) {
    self.visit_program(program);

    self.pass = Pass::Second;

    self.visit_program(program);

    let hash = {
      let mut hasher = DefaultHasher::new();
      self.file_path.hash(&mut hasher);
      hasher.finish()
    };
    let hash_string = format!("{:x}.css", hash);
    let import_path = format!("jsx-styled-vite-plugin/cached-css/{}", hash_string);
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

    let file = File::create(format!("{}/{}", self.css_out_dir, hash_string));
    if file.is_err() {
      eprintln!(
        "Failed to create file in css_out_dir: {}, reason {:#?}",
        self.css_out_dir,
        file.err()
      );
      return;
    }
    file
      .unwrap()
      .write_all(self.extracted_css.join("\n").as_bytes())
      .unwrap();

    program.body.insert(0, import_statement);
  }

  fn process_function_body(&mut self, body: &mut FunctionBody<'a>, fn_start: u32) {
    let mut has_style = false;
    let mut css: String = "".to_string();

    let mut style_detector = StyleDetector {
      found: &mut has_style,
      css: &mut css,
      scoping: self.scoping,
      style_tag_symbols: &self.style_tag_symbols,
    };

    style_detector.visit_function_body(&body);

    if has_style {
      let css_string = {
        if self.js_env.is_some() {
          if let Some(preprocessor) = &self.css_preprocessor {
            let input_js = self.js_env.as_ref().unwrap().create_string(&css).unwrap();
            let result = preprocessor.call(None, &[input_js]).unwrap();
            let returned_string = result
              .coerce_to_string()
              .unwrap()
              .into_utf8()
              .unwrap()
              .as_str()
              .unwrap()
              .to_owned();

            returned_string
          } else {
            css
          }
        } else {
          css
        }
      };

      let parsed_css = parse_css(&css_string).unwrap();
      let css_exports = parsed_css.exports.as_ref().unwrap();
      let identifier_symbol_ids: Vec<SymbolStore> = vec![];

      let mut classname_replacer = ClassNameReplacer {
        allocator: self.allocator,
        class_name_map: css_exports.clone(),
        ast_builder: self.ast_builder,
        scoping: self.scoping,
        identifier_symbol_ids: identifier_symbol_ids,
        fn_id: fn_start,
        classname_util_symbols: self.classname_util_symbols.clone(),
      };

      self
        .css_module_exports
        .insert(fn_start, css_exports.clone());

      classname_replacer.visit_function_body(body);

      self
        .identifier_symbol_ids
        .append(&mut classname_replacer.get_identifier_symbol_ids().to_vec());

      self.extracted_css.push(parsed_css.code);
    }
  }
}

impl<'a> VisitMut<'a> for TransformVisitor<'a> {
  fn leave_node(&mut self, kind: oxc_ast::AstType) {
    if self.pass == Pass::Second {
      return;
    }
    match kind {
      AstType::Function => {
        self.functions_stack.pop();
      }
      AstType::ArrowFunctionExpression => {
        self.functions_stack.pop();
      }
      _ => {}
    }
  }

  fn visit_import_declaration(&mut self, it: &mut oxc_ast::ast::ImportDeclaration<'a>) {
    if it.source.value == IMPORT_PATH {
      it.specifiers
        .as_ref()
        .unwrap()
        .iter()
        .for_each(|specifier| {
          if let ImportDeclarationSpecifier::ImportSpecifier(import_specifier) = specifier {
            if import_specifier.local.name == "Style" {
              self
                .style_tag_symbols
                .push(import_specifier.local.symbol_id());
            } else if import_specifier.local.name == "c" {
              self
                .classname_util_symbols
                .push(import_specifier.local.symbol_id());
            }
          }
        });
    }
  }
  fn visit_variable_declaration(&mut self, it: &mut oxc_ast::ast::VariableDeclaration<'a>) {
    if self.pass == Pass::First {
      it.declarations.iter().for_each(|decl| {
        if let Some(init) = &decl.init {
          match init {
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {
              if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
                self
                  .function_names
                  .push((ident.name.to_string(), ident.symbol_id()));

                self.functions_stack.push(ident.symbol_id());
              }
            }
            _ => {}
          }
        }
      });

      return walk_mut::walk_variable_declaration(self, it);
    }

    it.declarations.iter_mut().for_each(|decl| {
      if let BindingPatternKind::BindingIdentifier(binding_identifier) = &decl.id.kind {
        let symbol_id = binding_identifier.symbol_id();
        let symbol_store_item = self
          .identifier_symbol_ids
          .iter()
          .find(|s| s.symbol_id == symbol_id);

        if symbol_store_item.is_some() {
          let css_exports = self
            .css_module_exports
            .get(&symbol_store_item.unwrap().fn_id);
          if css_exports.is_none() {
            return;
          }
          let css_exports = css_exports.unwrap();
          let identifier_symbol_ids: Vec<SymbolStore> = vec![];

          let mut classname_replacer = ClassNameReplacer {
            allocator: self.allocator,
            class_name_map: css_exports.clone(),
            ast_builder: self.ast_builder,
            scoping: self.scoping,
            identifier_symbol_ids: identifier_symbol_ids,
            fn_id: decl.span.start,
            classname_util_symbols: self.classname_util_symbols.clone(),
          };

          if decl.init.is_some() {
            classname_replacer.update_expression(decl.init.as_mut());
          }
        }
      }
    });
  }
  fn visit_arrow_function_expression(
    &mut self,
    it: &mut oxc_ast::ast::ArrowFunctionExpression<'a>,
  ) {
    if self.pass == Pass::Second {
      return walk_mut::walk_arrow_function_expression(self, it);
    }

    let body = it.body.as_mut();
    self.process_function_body(body, it.span.start);

    walk_mut::walk_arrow_function_expression(self, it);
  }
  fn visit_function(&mut self, function: &mut Function<'a>, flags: ScopeFlags) {
    if self.pass == Pass::Second {
      return walk_mut::walk_function(self, function, flags);
    }

    let func_id = function.id.as_ref().unwrap();
    self
      .function_names
      .push((func_id.name.to_string(), func_id.symbol_id()));

    self.functions_stack.push(func_id.symbol_id());

    let body = function.body.as_mut().unwrap();
    self.process_function_body(body, function.span.start);

    walk_mut::walk_function(self, function, flags);
  }

  fn visit_jsx_element(&mut self, jsx: &mut JSXElement<'a>) {
    let name = &jsx.opening_element.name;

    if let JSXElementName::IdentifierReference(ident) = name {
      let reference = self.scoping.get_reference(ident.reference_id());
      let symbol_id = reference.symbol_id().unwrap();
      if self.style_tag_symbols.contains(&symbol_id) {
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

        if let Some(last_function) = self.functions_stack.last() {
          self
            .function_to_style_mapping
            .insert(last_function.clone(), vec![extracted_css]);
        }
      }
    }
  }

  fn visit_expression(&mut self, it: &mut oxc_ast::ast::Expression<'a>) {
    if self.pass != Pass::First {
      return walk_mut::walk_expression(self, it);
    }
    if let Expression::AssignmentExpression(assign) = &it {
      if let AssignmentTarget::StaticMemberExpression(static_member) = &assign.left {
        if let Expression::Identifier(ident) = &static_member.object {
          let reference = ident.reference_id();
          let symbol_id = self.scoping.get_reference(reference).symbol_id().unwrap();
          if self
            .function_names
            .contains(&(ident.name.to_string(), symbol_id))
            && &static_member.property.name.to_string() == "flair"
          {
            let content = &assign.right;
            let mut extracted_css = String::new();
            if let Expression::StringLiteral(string_value) = content {
              extracted_css.push_str(&string_value.value.to_string());
            } else if let Expression::TemplateLiteral(template_expression) = content {
              let template_expression_value = template_expression
                .quasis
                .iter()
                .map(|elem| elem.value.clone().raw.into_string())
                .collect::<Vec<String>>()
                .join("");

              extracted_css.push_str(&template_expression_value);

              if self.function_to_style_mapping.contains_key(&symbol_id) {
                self
                  .function_to_style_mapping
                  .get_mut(&symbol_id)
                  .unwrap()
                  .push(extracted_css);
              } else {
                self
                  .function_to_style_mapping
                  .insert(symbol_id, vec![extracted_css]);
              }
            }
          }
        }
      }
    }
    walk_mut::walk_expression(self, it);
  }
}

pub struct StyleDetector<'a> {
  found: &'a mut bool,
  css: &'a mut String,
  scoping: &'a Scoping,
  style_tag_symbols: &'a Vec<SymbolId>,
}

impl<'a> Visit<'_> for StyleDetector<'a> {
  fn visit_jsx_element(&mut self, jsx: &JSXElement<'_>) {
    let name = &jsx.opening_element.name;

    if let JSXElementName::IdentifierReference(ident) = name {
      let reference = self.scoping.get_reference(ident.reference_id());
      let symbol_id = reference.symbol_id().unwrap();
      if self.style_tag_symbols.contains(&symbol_id) {
        *self.found = true;

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
        *self.css = extracted_css;
      }
    }
  }
}

fn get_string_hash(s: &str) -> String {
  let mut hasher = DefaultHasher::new();
  s.hash(&mut hasher);
  let hash = hasher.finish();
  format!("{:x}", hash) // Converts the u64 hash to a hexadecimal string
}
