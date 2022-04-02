#![feature(prelude_import)]
//! This example shows how to wrap a data structure in a mutex to achieve safe mutability.
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
extern crate lazy_static;
use lazy_static::lazy_static;
#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct COMPILED_REGEX {
    __private_field: (),
}
#[doc(hidden)]
static COMPILED_REGEX: COMPILED_REGEX = COMPILED_REGEX {
    __private_field: (),
};
impl ::lazy_static::__Deref for COMPILED_REGEX {
    type Target = regex::Regex;
    fn deref(&self) -> &regex::Regex {
        #[inline(always)]
        fn __static_ref_initialize() -> regex::Regex {
            regex::Regex::new(".*").unwrap()
        }
        #[inline(always)]
        fn __stability() -> &'static regex::Regex {
            static LAZY: ::lazy_static::lazy::Lazy<regex::Regex> = ::lazy_static::lazy::Lazy::INIT;
            LAZY.get(__static_ref_initialize)
        }
        __stability()
    }
}
impl ::lazy_static::LazyStatic for COMPILED_REGEX {
    fn initialize(lazy: &Self) {
        let _ = &**lazy;
    }
}
fn main() {
    let _x = COMPILED_REGEX.is_match("abc");
}