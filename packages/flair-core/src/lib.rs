#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

pub mod transform;
pub mod parse_css;
pub mod update_attribute;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}
