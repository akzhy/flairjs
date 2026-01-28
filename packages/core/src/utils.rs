use napi_derive::napi;
use oxc_data_structures::rope::Rope;

pub struct ExtendedRope<'a> {
  rope: Rope,
  source_text: &'a str,
}

impl<'a> ExtendedRope<'a> {
  pub fn new(source_text: &'a str) -> Self {
    let rope = Rope::from_str(source_text);
    Self { rope, source_text }
  }

  pub fn get_line_column(&self, offset: u32) -> (u32, u32) {
    let offset = offset as usize;
    // Get line number and byte offset of start of line
    let line_index = self.rope.byte_to_line(offset);
    let line_offset = self.rope.line_to_byte(line_index);
    // Get column number
    let column_index = self.source_text[line_offset..offset].encode_utf16().count();
    ((line_index + 1) as u32, (column_index + 1) as u32)
  }
}

#[napi(object)]
pub struct UnusedCss {
  pub class_name: String,
  pub line: u32,
  pub column: u32,
}