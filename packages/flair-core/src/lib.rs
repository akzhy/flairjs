#![deny(clippy::all)]

use crate::transform::TransformOutput;

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
pub fn transform_code(code: String, file_path: String) -> Option<TransformOutput> {
  let options = transform::TransformOptions {
    code,
    file_path,
    css_preprocessor: None, // Pass a preprocessor function if needed
  };
  
  transform::transform(options)
}