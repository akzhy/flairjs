use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;

use crate::flair_property::FlairProperty;
use crate::style_tag::StyleDetector;
use crate::update_attribute::ClassNameReplacer;
use crate::{parse_css::parse_css, update_attribute::SymbolStore};
use lightningcss::css_modules::CssModuleExport;
use napi::bindgen_prelude::Function as JsFunction;
use napi::Env;
use napi_derive::napi;
use oxc::ast::ast::{
  ArrowFunctionExpression, BindingPatternKind, FunctionBody, ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind, JSXChild, Statement, VariableDeclaration
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

#[derive(PartialEq, Debug, Clone)]
enum Pass {
  First,
  Second,
  Third,
}

const IMPORT_PATH: &str = "@flairjs/react";

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
  css_preprocessor: Option<JsFunction<String, String>>,
  env: Option<Env>,
) -> Option<TransformOutput> {
  if !options.file_path.ends_with(".tsx") {
    return None;
  }

  let allocator = Allocator::default();
  let source_type = SourceType::from_path(&options.file_path).unwrap();

  let sourcemap_file_path = options.file_path.clone();

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
  style_tag_import_symbols: Vec<SymbolId>,
  classname_util_symbols: Vec<SymbolId>,
  // local_class_name_util_functions: &'a mut Vec<String>,
  extracted_css: Vec<String>,
  css_preprocessor: &'a Option<JsFunction<'a, String, String>>,
  ast_builder: AstBuilder<'a>,
  scoping: &'a Scoping,
  identifier_symbol_ids: Vec<SymbolStore>,
  pass: Pass,
  css_module_exports: HashMap<u32, HashMap<String, CssModuleExport>>,
  file_path: String,
  css_out_dir: String,
  js_env: Option<Env>,
  function_id_to_raw_css_mapping: HashMap<u32, Vec<String>>,
  variable_linking: HashMap<SymbolId, SymbolId>,
  flair_property_visitor: FlairProperty<'a>,
  style_tag_symbols: Vec<u32>,
}

impl<'a> TransformVisitor<'a> {
  fn new(
    allocator: &'a Allocator,
    css_preprocessor: &'a Option<JsFunction<'a, String, String>>,
    ast_builder: AstBuilder<'a>,
    scoping: &'a Scoping,
    file_path: String,
    css_out_dir: String,
    js_env: Option<Env>,
  ) -> Self {
    let extracted_css = vec![];
    let identifier_symbol_ids: Vec<SymbolStore> = vec![];
    let css_module_exports: HashMap<u32, HashMap<String, CssModuleExport>> = HashMap::new();
    let style_tag_symbols: Vec<u32> = vec![];
    let style_tag_import_symbols: Vec<SymbolId> = vec![];
    let classname_util_symbols: Vec<SymbolId> = vec![];

    let function_id_to_style_mapping = HashMap::new();
    let variable_linking = HashMap::new();
    let flair_property_visitor = FlairProperty::new(&scoping, &allocator);

    Self {
      allocator,
      style_tag_import_symbols,
      style_tag_symbols,
      classname_util_symbols,
      extracted_css,
      variable_linking,
      css_preprocessor,
      ast_builder,
      scoping,
      identifier_symbol_ids,
      pass: Pass::First,
      css_module_exports,
      file_path,
      css_out_dir,
      js_env,
      function_id_to_raw_css_mapping: function_id_to_style_mapping,
      flair_property_visitor,
    }
  }

  fn begin(&mut self, program: &mut Program<'a>) {
    self.visit_program(program);

    let flair_styles = self.flair_property_visitor.get_style();

    flair_styles.iter().for_each(|(span_start, style)| {
      if self.function_id_to_raw_css_mapping.contains_key(span_start) {
        self
          .function_id_to_raw_css_mapping
          .get_mut(span_start)
          .unwrap()
          .push(style.to_string());
        return;
      }

      self
        .function_id_to_raw_css_mapping
        .insert(*span_start, vec![style.to_string()]);
    });

    self
      .function_id_to_raw_css_mapping
      .iter()
      .for_each(|(fn_id, styles)| {
        let css = styles.join("\n");

        let css_string = {
          if self.js_env.is_some() {
            if let Some(preprocessor) = &self.css_preprocessor {
              let result = preprocessor.call(css).unwrap();
              result
            } else {
              css.to_string()
            }
          } else {
            css.to_string()
          }
        };

        let parsed_css = parse_css(&css_string).unwrap();
        let css_exports = parsed_css.exports.as_ref().unwrap();

        self.css_module_exports.insert(*fn_id, css_exports.clone());

        self.extracted_css.push(parsed_css.code);
      });

    // //   let parsed_css = parse_css(&css_string).unwrap();
    // //   let css_exports = parsed_css.exports.as_ref().unwrap();

    self.pass = Pass::Second;

    self.visit_program(program);

    self.pass = Pass::Third;
    self.visit_program(program);

    let hash = {
      let mut hasher = DefaultHasher::new();
      self.file_path.hash(&mut hasher);
      hasher.finish()
    };

    let hash_string = format!("{:x}.css", hash);
    let import_path = format!("@flairjs/css/cached-css/{}", hash_string);
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
    match self.pass {
      Pass::First => {
        let mut style_detector = StyleDetector::new(&self.scoping, &self.style_tag_import_symbols);
        style_detector.visit_function_body(&body);

        if style_detector.has_style {
          self.style_tag_symbols = style_detector.get_style_tag_symbol_ids();
          self
            .function_id_to_raw_css_mapping
            .insert(fn_start, style_detector.css);
        }
      }
      Pass::Second => {
        let mut classname_replacer = ClassNameReplacer {
          allocator: self.allocator,
          class_name_map: self
            .css_module_exports
            .get(&fn_start)
            .cloned()
            .unwrap_or_default(),
          ast_builder: self.ast_builder,
          scoping: self.scoping,
          identifier_symbol_ids: self.identifier_symbol_ids.clone(),
          fn_id: fn_start,
          classname_util_symbols: self.classname_util_symbols.clone(),
          variable_linking: self.variable_linking.clone(),
        };

        classname_replacer.visit_function_body(body);

        self.identifier_symbol_ids = classname_replacer.get_identifier_symbol_ids().to_vec();
      }
      Pass::Third => {}
    }
  }
}

impl<'a> VisitMut<'a> for TransformVisitor<'a> {
  fn visit_import_declaration(&mut self, it: &mut ImportDeclaration<'a>) {
    if it.source.value == IMPORT_PATH {
      it.specifiers
        .as_ref()
        .unwrap()
        .iter()
        .for_each(|specifier| {
          if let ImportDeclarationSpecifier::ImportSpecifier(import_specifier) = specifier {
            if import_specifier.local.name == "Style" {
              self
                .style_tag_import_symbols
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

  fn visit_variable_declaration(&mut self, it: &mut VariableDeclaration<'a>) {
    match self.pass {
      Pass::First => {
        it.declarations.iter_mut().for_each(|decl| {
          if let BindingPatternKind::BindingIdentifier(binding_identifier) = &decl.id.kind {
            if let Some(init) = &decl.init {
              if let Expression::Identifier(identifier) = init {
                let symbol_id = binding_identifier.symbol_id();
                let reference = self.scoping.get_reference(identifier.reference_id());
                let init_symbol_id = reference.symbol_id();

                if let Some(init_symbol_id) = init_symbol_id {
                  self.variable_linking.insert(symbol_id, init_symbol_id);
                }
              }
            }
          }
        });

        self.flair_property_visitor.visit_variable_declaration(it);
      }
      Pass::Second => {}
      Pass::Third => {
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
                variable_linking: self.variable_linking.clone(),
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

  fn visit_expression(&mut self, it: &mut Expression<'a>) {
    if self.pass == Pass::First {
      self.flair_property_visitor.visit_expression(it);
    } else if self.pass == Pass::Second {
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

  fn visit_arrow_function_expression(
    &mut self,
    it: &mut ArrowFunctionExpression<'a>,
  ) {
    let body = it.body.as_mut();
    self.process_function_body(body, it.span.start);

    walk_mut::walk_arrow_function_expression(self, it);
  }

  fn visit_function(&mut self, function: &mut Function<'a>, flags: ScopeFlags) {
    let body = function.body.as_mut().unwrap();
    self.process_function_body(body, function.span.start);
    self.flair_property_visitor.visit_function(function);

    walk_mut::walk_function(self, function, flags);
  }
}
