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
use oxc_ast::ast::Program;
use oxc_ast::ast::{
  BindingPatternKind, FunctionBody, ImportDeclarationSpecifier, ImportOrExportKind, Statement,
};
use oxc_ast::NONE;
use oxc_ast::{ast::Function, AstBuilder};
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
  function_id_to_raw_css_mapping: HashMap<u32, Vec<String>>,
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

    let function_id_to_style_mapping = HashMap::new();

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
      function_id_to_raw_css_mapping: function_id_to_style_mapping,
    }
  }

  fn begin(&mut self, program: &mut Program<'a>) {
    self.visit_program(program);

    let mut flair_property_visitor = FlairProperty::new(&self.scoping);
    flair_property_visitor.visit_program(program);

    let flair_styles = flair_property_visitor.get_style();

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

    // self.pass = Pass::Second;

    // self.visit_program(program);

    // self.pass = Pass::Third;
    // self.visit_program(program);
    println!("Extracted CSS: {:?}", self.extracted_css);

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
    match self.pass {
      Pass::First => {
        let mut style_detector = StyleDetector::new(&self.scoping, &self.style_tag_symbols);
        style_detector.visit_function_body(&body);

        if style_detector.has_style {
          self
            .function_id_to_raw_css_mapping
            .insert(fn_start, style_detector.css);
        }
      }
      _ => {}
    }
  }
}

impl<'a> VisitMut<'a> for TransformVisitor<'a> {
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

    let body = function.body.as_mut().unwrap();
    self.process_function_body(body, function.span.start);

    walk_mut::walk_function(self, function, flags);
  }
}
