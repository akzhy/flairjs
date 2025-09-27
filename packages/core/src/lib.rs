#![deny(clippy::all)]

use std::time::Instant;

use napi::bindgen_prelude::{Env, Function};

use crate::transform::{TransformOptions, TransformOutput};

#[macro_use]
extern crate napi_derive;

pub mod flair_property;
pub mod logger;
pub mod parse_css;
pub mod style_tag;
pub mod transform;
pub mod update_attribute;

pub use crate::logger::{log_error, log_info, log_warn};

#[napi]
pub fn transform_code(
  env: Env,
  code: String,
  file_path: String,
  options: TransformOptions,
  css_preprocessor: Option<Function<String, String>>,
) -> Option<TransformOutput> {
  let time = Instant::now();

  // Example of using the logging system
  println!("Starting transformation for file: {}", file_path);

  let options = transform::TransformOptions {
    css_out_dir: options.css_out_dir,
    class_name_list: options.class_name_list,
    use_theme: options.use_theme,
    append_timestamp_to_css_file: options.append_timestamp_to_css_file,
    theme: options.theme,
  };
  let result = transform::transform(code, file_path, options, css_preprocessor, Some(env));
  let duration = time.elapsed();

  println!("Transformation completed in {:?}", duration);

  result
}
