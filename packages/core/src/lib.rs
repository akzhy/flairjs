#![deny(clippy::all)]

use std::time::Instant;

use napi::{bindgen_prelude::Function, Env };

use crate::transform::{TransformOptions, TransformOutput};

#[macro_use]
extern crate napi_derive;

pub mod flair_property;
pub mod parse_css;
pub mod style_tag;
pub mod transform;
pub mod update_attribute;


#[napi]
pub fn transform_code(
  env: Env,
  options: TransformOptions,
  css_preprocessor: Option<Function<String, String>>,
) -> Option<TransformOutput> {
  let time = Instant::now();
  println!("Starting transformation...");
  let options = transform::TransformOptions {
    code: options.code,
    file_path: options.file_path,
    css_out_dir: options.css_out_dir,
  };
  let result = transform::transform(options, css_preprocessor, Some(env));
  let duration = time.elapsed();
  println!("Transformation completed in {:?}", duration);
  result
}
