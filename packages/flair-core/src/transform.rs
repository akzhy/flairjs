use crate::parse_css::parse_css;
use crate::update_attribute::AttributeUpdater;
use oxc::{
  allocator::Allocator,
  ast::ast::SourceType,
  ast_visit::{walk_mut, Visit, VisitMut},
  parser::{Parser, ParserReturn},
  semantic::{ScopeFlags, Scoping, SemanticBuilder, SymbolId},
  syntax::identifier,
};
use oxc_ast::ast::{
  BindingIdentifier, BindingPatternKind, JSXChild, JSXElementName, JSXExpression,
};
use oxc_ast::ast::{JSXElement, Program};
use oxc_ast::{ast::Function, AstBuilder};
use oxc_codegen::{Codegen, CodegenOptions};

#[derive(PartialEq, Debug, Clone)]
enum Pass {
  First,
  Second,
}

pub struct TransformOptions {
  pub code: String,
  pub file_path: String,
  pub css_preprocessor: Option<Box<dyn Fn(String, String) -> String>>,
  pub output_type: Option<String>, // "inject-import" or "write-css-file"
  pub output_path: Option<String>,
}

pub fn transform(options: TransformOptions) -> Option<String> {
  if !options.file_path.ends_with(".tsx") {
    return None;
  }

  let allocator = Allocator::default();
  let source_type = SourceType::from_path(&options.file_path).unwrap();

  let ParserReturn { mut program, .. } =
    Parser::new(&allocator, &options.code, source_type).parse();

  let semantic_builder = SemanticBuilder::new().build(&program);

  let scoping = semantic_builder.semantic.into_scoping();

  let mut local_style_tag_name = "style".to_string();
  let mut local_class_name_util_functions = vec![];
  let mut extracted_css = vec![];
  let ast_builder = AstBuilder::new(&allocator);
  let mut symbold_ids_vec: Vec<SymbolId> = vec![];

  // Traverse the AST
  let mut visitor = TransformVisitor {
    allocator: &allocator,
    local_style_tag_name: &mut local_style_tag_name,
    local_class_name_util_functions: &mut local_class_name_util_functions,
    extracted_css: &mut extracted_css,
    file_path: &options.file_path,
    css_preprocessor: &options.css_preprocessor,
    output_type: options.output_type.clone(),
    ast_builder,
    scoping: &scoping,
    identifier_symbol_ids: &mut symbold_ids_vec,
    pass: Pass::First,
  };

  visitor.begin(&mut program);

  let codegen = Codegen::new();
  let result = codegen.build(&program);

  // println!("Transformed code: {}", result.code);
  // println!("Program: {:#?}", program);

  //   if let Some(output_type) = options.output_type {
  //     if output_type == "write-css-file" {
  //       if let Some(output_path) = options.output_path {
  //         // let css = extracted_css.join("\n");
  //         // fs::write(output_path, css).expect("Failed to write CSS file");
  //       }
  //     }
  //   }
  let result: String = "done".to_string();
  Some(result)
}

struct TransformVisitor<'a> {
  allocator: &'a Allocator,
  local_style_tag_name: &'a mut String,
  local_class_name_util_functions: &'a mut Vec<String>,
  extracted_css: &'a mut Vec<String>,
  file_path: &'a str,
  css_preprocessor: &'a Option<Box<dyn Fn(String, String) -> String>>,
  output_type: Option<String>,
  ast_builder: AstBuilder<'a>,
  scoping: &'a Scoping,
  identifier_symbol_ids: &'a mut Vec<SymbolId>,
  pass: Pass,
}

impl<'a> TransformVisitor<'a> {
  fn begin(&mut self, program: &mut Program<'a>) {
    self.visit_program(program);

    self.pass = Pass::Second;

    // println!("Identifiers {:#?}", self.identifier_symbol_ids);
    self.visit_program(program);
  }
}

impl<'a> VisitMut<'a> for TransformVisitor<'a> {
  fn visit_variable_declaration(&mut self, it: &mut oxc_ast::ast::VariableDeclaration<'a>) {
    if self.pass == Pass::First {
      return walk_mut::walk_variable_declaration(self, it);
    }

    it.declarations.iter().for_each(|decl| {
      if let BindingPatternKind::BindingIdentifier(binding_identifier) = &decl.id.kind {
        let symbold_id = binding_identifier.symbol_id();
        if self.identifier_symbol_ids.contains(&symbold_id) {
          println!(
            "Symbol ID already exists: {:#?}, {:#?}, {:#?}",
            self.identifier_symbol_ids, symbold_id, binding_identifier
          );
        }
      }
    });
  }
  fn visit_arrow_function_expression(
    &mut self,
    it: &mut oxc_ast::ast::ArrowFunctionExpression<'a>,
  ) {
    let mut has_style = false;
    let mut css: String = "".to_string();

    let mut style_detector = StyleDetector {
      found: &mut has_style,
      css: &mut css,
    };

    let body = it.body.as_ref();
    style_detector.visit_function_body(&body);

    if has_style {
      // println!("Style found in arrow function");
    }
  }
  fn visit_function(&mut self, function: &mut Function<'a>, flags: ScopeFlags) {
    println!("Pass id is: {:?}", self.pass);
    if self.pass == Pass::Second {
      return walk_mut::walk_function(self, function, flags);
    }

    let mut has_style = false;
    let mut css: String = "".to_string();

    let mut style_detector = StyleDetector {
      found: &mut has_style,
      css: &mut css,
    };

    let body = function.body.as_mut().unwrap();

    style_detector.visit_function_body(&body);

    if has_style {
      let parsed_css = parse_css(&css).unwrap();
      let identifier_symbol_ids: Vec<SymbolId> = vec![];

      let mut attribute_updater = AttributeUpdater {
        allocator: self.allocator,
        class_name_map: parsed_css.exports.unwrap(),
        ast_builder: self.ast_builder,
        scoping: self.scoping,
        identifier_symbol_ids: identifier_symbol_ids,
      };

      attribute_updater.visit_function_body(body);

      self
        .identifier_symbol_ids
        .append(&mut attribute_updater.get_identifier_symbol_ids().to_vec());
    }
    walk_mut::walk_function(self, function, flags);
  }
}

pub struct StyleDetector<'a> {
  found: &'a mut bool,
  css: &'a mut String,
}

impl<'a> Visit<'_> for StyleDetector<'a> {
  fn visit_jsx_element(&mut self, jsx: &JSXElement<'_>) {
    let name = &jsx.opening_element.name;

    if let JSXElementName::IdentifierReference(ident) = name {
      if ident.name == "Style" {
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
