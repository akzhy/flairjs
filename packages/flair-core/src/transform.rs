use crate::parse_css::parse_css;
use crate::update_attribute::AttributeUpdater;
use oxc::{
  allocator::Allocator,
  ast::ast::SourceType,
  ast_visit::{Visit, VisitMut},
  parser::{Parser, ParserReturn},
  semantic::ScopeFlags,
};
use oxc_ast::{ast::Function, AstBuilder};
use oxc_ast::ast::JSXElement;
use oxc_ast::ast::{JSXChild, JSXElementName, JSXExpression};
use oxc_codegen::{Codegen, CodegenOptions};

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

  let mut local_style_tag_name = "style".to_string();
  let mut local_class_name_util_functions = vec![];
  let mut extracted_css = vec![];
  let ast_builder = AstBuilder::new(&allocator);

  // Traverse the AST
  let mut visitor = TransformVisitor {
    allocator: &allocator,
    local_style_tag_name: &mut local_style_tag_name,
    local_class_name_util_functions: &mut local_class_name_util_functions,
    extracted_css: &mut extracted_css,
    file_path: &options.file_path,
    css_preprocessor: &options.css_preprocessor,
    output_type: options.output_type.clone(),
    ast_builder
  };

  visitor.visit_program(&mut program);

  let codegen = Codegen::new();
  let result = codegen.build(&program);

  println!("Transformed code: {}", result.code);
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
}

impl<'a> VisitMut<'_> for TransformVisitor<'a> {
  fn visit_arrow_function_expression(
    &mut self,
    it: &mut oxc_ast::ast::ArrowFunctionExpression<'_>,
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
      println!("Style found in arrow function");
    }
  }
  fn visit_function(&mut self, function: &mut Function<'_>, _flags: ScopeFlags) {
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

      let mut attribute_updater = AttributeUpdater {
        allocator: self.allocator,
        class_name_map: parsed_css.exports.unwrap(),
        ast_builder: self.ast_builder,
      };

      attribute_updater.visit_function_body(body);
    }
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
