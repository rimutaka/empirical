//! This example shows how to wrap a data structure in a mutex to achieve safe mutability.
extern crate lazy_static;
use lazy_static::lazy_static;

lazy_static! {
    static ref COMPILED_REGEX: regex::Regex = regex::Regex::new(".*").unwrap();
}

fn main() {
    let _x = COMPILED_REGEX.is_match("abc");
}
