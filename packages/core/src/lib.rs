#![deny(clippy::all)]

use std::time::Instant;

use napi::bindgen_prelude::Env;

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
  code: String,
  file_path: String,
  options: TransformOptions,
) -> Option<TransformOutput> {
  let time = Instant::now();
  println!("Starting transformation...");
  let options = transform::TransformOptions {
    css_out_dir: options.css_out_dir,
    classname_list: options.classname_list,
    css_preprocessor: options.css_preprocessor,
  };
  let result = transform::transform(code, file_path, options, Some(env));
  let duration = time.elapsed();
  println!("Transformation completed in {:?}", duration);
  result
}
