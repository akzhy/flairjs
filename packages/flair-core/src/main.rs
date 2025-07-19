mod transform;
mod parse_css;
mod update_attribute;
use std::{fs, time::Instant};
use transform::{transform, TransformOptions};

fn main() {
  let contents = fs::read_to_string("src/App.tsx").expect("Something went wrong reading the file");
  
  let start = Instant::now();
  
  let options = TransformOptions {
    code: contents,
    file_path: "App.tsx".to_string(),
    css_preprocessor: None, // Pass a preprocessor function if needed
  };

  let duration = start.elapsed();

  if let Some(result) = transform(options) {
    println!("Transformed code:\n{}", result.code);
    println!("Transformation took: {:?}", duration);
  } else {
    println!("Transformation failed or skipped.");
  }
}
