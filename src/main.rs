// This file contains code taken from https://github.com/rust-lang-nursery/lazy-static.rs
// Their copyright statement is copied below.
//
// Copyright 2016 lazy-static.rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::ops::Deref;
use std::cell::Cell;
use std::sync::Once;

/// A test regex to validate an email address - used here for a test
const LONG_REGEX: &str = r#"[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*@(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?"#;
/// A valid email for a test
const TEST_EMAIL: &str = "name@example.com";
/// An invalid email for
const TEST_NOT_EMAIL: &str = "Hello world!";

/// A container for lazy initialization of a compiled version of LONG_REGEX
struct Lazy<T: Sync>(Cell<Option<T>>, Once);

// This line is required because
// `Cell<Option<regex::Regex>>` cannot be shared between threads safely within `Lazy<regex::Regex>`, the trait `Sync` is not implemented for `Cell<Option<regex::Regex>>`
// Shared static variables must have a type that implements `Sync`rustc [E0277](https://doc.rust-lang.org/error-index.html#E0277)
unsafe impl<T: Sync> Sync for Lazy<T> {}

/// A structure with a hidden variable to store the compiled regex
struct CompiledRegex {
    __private_field: (),
}

/// The actual lazily-initialized variable with the compiled regex
static COMPILED_REGEX: CompiledRegex = CompiledRegex {
    __private_field: (),
};

// This Deref implementation initializes the hidden variable with the compiled regex on the first run
// and returns its value on subsequent runs.
impl Deref for CompiledRegex {
    type Target = regex::Regex;
    fn deref(&self) -> &regex::Regex {
        static LAZY: Lazy<regex::Regex> = Lazy(Cell::new(None), Once::new());

        println!("Derefencing CompiledRegex");

        // Performs an initialization routine once and only once.
        // The given closure will be executed if this is the first time call_once has been called, and otherwise the routine will not be invoked.
        // https://doc.rust-lang.org/std/sync/struct.Once.html#method.call_once
        LAZY.1.call_once(|| {
            LAZY.0.set(Some(regex::Regex::new(LONG_REGEX).unwrap()));
            println!("CompiledRegex initialized");
        });

        // `self.0` is guaranteed to be `Some` by this point
        // The `Once` will catch and propagate panics
        unsafe {
            match *LAZY.0.as_ptr() {
                Some(ref x) => x,
                None => {
                    panic!("attempted to dereference an uninitialized lazy static. This is a bug");
                }
            }
        }
    }
}

fn main() {
    // at this point COMPILED_REGEX is not initialized
    println!("Program started");

    // COMPILED_REGEX will be initialized during this call
    println!(
        "{TEST_EMAIL} is valid: {}",
        COMPILED_REGEX.is_match(TEST_EMAIL)
    );

    // a previously initialized COMPILED_REGEX is used
    println!(
        "{TEST_NOT_EMAIL} is valid: {}",
        COMPILED_REGEX.is_match(TEST_NOT_EMAIL)
    );
}
