#![deny(clippy::all)]

use napi::{Env, JsFunction};

use crate::transform::{TransformOptions, TransformOutput};

#[macro_use]
extern crate napi_derive;

pub mod transform;
pub mod parse_css;
pub mod update_attribute;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

#[napi]
pub fn transform_code(env: Env, options: TransformOptions, css_preprocessor: Option<JsFunction>) -> Option<TransformOutput> {
  let options = transform::TransformOptions {
    code: options.code,
    file_path: options.file_path,
    css_out_dir: options.css_out_dir,
  };
  
  transform::transform(options, css_preprocessor, Some(env))
}