#![deny(clippy::all)]

use std::time::Instant;

use napi::bindgen_prelude::{Env, Function};

use crate::transform::{CSSData, TransformOptions, TransformOutput};

#[macro_use]
extern crate napi_derive;

pub mod flair_property;
pub mod logger;
pub mod parse_css;
pub mod style_tag;
pub mod transform;
pub mod update_attribute;
pub mod utils;

pub use crate::logger::{log_error, log_info, log_warn};

#[napi]
pub fn transform_code(
  env: Env,
  code: String,
  file_path: String,
  options: TransformOptions,
  css_preprocessor: Option<Function<CSSData, String>>,
) -> TransformOutput {
  let time = Instant::now();

  // Example of using the logging system
  if cfg!(debug_assertions) {
    println!("Starting transformation for file: {}", file_path);
  }

  let result = transform::transform(code, file_path, options, css_preprocessor, Some(env));
  let duration = time.elapsed();

  if cfg!(debug_assertions) {
    println!("Transformation completed in {:?}", duration);
  }

  result
}
