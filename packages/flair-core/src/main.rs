mod transform;
mod parse_css;
mod update_attribute;
use std::{fs, time::Instant};
use transform::{transform, TransformOptions};

fn main() {
  let contents = fs::read_to_string("src/TestCase.tsx").expect("Something went wrong reading the file");
  
  let start = Instant::now();
  
  let options = TransformOptions {
    code: contents,
    file_path: "TestCase.tsx".to_string(),
    css_out_dir: ".cache/css_output".to_string(), // Specify your CSS output directory
  };

  let duration = start.elapsed();

  if let Some(result) = transform(options, None, None) {
    println!("Transformed code:\n{}", result.code);
    println!("Transformation took: {:?}", duration);
  } else {
    println!("Transformation failed or skipped.");
  }
}
