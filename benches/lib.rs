#![feature(test)]

extern crate test;
use test::Bencher;

#[macro_use]
extern crate lazy_static;

mod external_mod;

/// Finds email addresses. Taken from https://github.com/rust-lang/regex/blob/master/tests/crazy.rs
pub(crate) const LONG_REGEX: &str = r#"[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*@(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?"#;
pub(crate) const TEST_EMAIL: &str = "max@example.com";

lazy_static! {
    pub(crate) static ref COMPILED_REGEX: regex::Regex = regex::Regex::new(LONG_REGEX).unwrap();
}

static COMPILED_REGEX_ONCE_CELL: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(LONG_REGEX).unwrap());

/// The regex is compiled within lazy_static at the module level
#[bench]
fn lazy_static_local(b: &mut Bencher) {
    b.iter(|| {
        let is_match = COMPILED_REGEX.is_match(TEST_EMAIL);
        test::black_box(is_match);
    });
}

#[test]
fn lazy_static_local_test() {
    let is_match = COMPILED_REGEX.is_match(TEST_EMAIL);
    assert!(is_match);
}

/// The regex is compiled once within the bench function
#[bench]
fn vanilla_rust_local(b: &mut Bencher) {
    let compiled_regex = regex::Regex::new(LONG_REGEX).unwrap();
    b.iter(|| {
        let is_match = compiled_regex.is_match(TEST_EMAIL);
        test::black_box(is_match);
    });
}

#[test]
fn vanilla_rust_local_test() {
    let compiled_regex = regex::Regex::new(LONG_REGEX).unwrap();
    let is_match = compiled_regex.is_match(TEST_EMAIL);
    assert!(is_match);
}

/// The regex is compiled once by once_cell::sync::Lazy on the first use within the loop
#[bench]
fn once_cell_lazy(b: &mut Bencher) {
    b.iter(|| {
        let is_match = COMPILED_REGEX_ONCE_CELL.is_match(TEST_EMAIL);
        test::black_box(is_match);
    });
}

#[test]
fn once_cell_lazy_test() {
    let is_match = COMPILED_REGEX_ONCE_CELL.is_match(TEST_EMAIL);
    assert!(is_match);
}

/// The regex is compiled within the bench function loop, which is obviously inefficient,
/// but we do it to show the cost of its compilation
#[bench]
fn bad_rust_local(b: &mut Bencher) {
    b.iter(|| {
        let compiled_regex = regex::Regex::new(LONG_REGEX).unwrap();
        let is_match = compiled_regex.is_match(TEST_EMAIL);
        test::black_box(is_match);
    });
}

#[test]
fn bad_rust_local_test() {
    let compiled_regex = regex::Regex::new(LONG_REGEX).unwrap();
    let is_match = compiled_regex.is_match(TEST_EMAIL);
    assert!(is_match);
}

/// The regex is compiled within lazy_static declared in a separate module
/// and used by a function within that module called repeatedly within the loop
#[bench]
fn lazy_static_inner(b: &mut Bencher) {
    b.iter(|| {
        let is_match = inner::lazy_static_local(TEST_EMAIL);
        test::black_box(is_match);
    });
}

#[test]
fn lazy_static_inner_test() {
    let is_match = inner::lazy_static_local(TEST_EMAIL);
    assert!(is_match);
}

/// The regex is compiled within lazy_static declared in a separate module
/// placed in an external file. It is expected to be compiled once only.
#[bench]
fn lazy_static_external_mod(b: &mut Bencher) {
    b.iter(|| {
        let is_match = external_mod::lazy_static_external(TEST_EMAIL);
        test::black_box(is_match);
    });
}

#[test]
fn lazy_static_external_mod_test() {
    let is_match = external_mod::lazy_static_external(TEST_EMAIL);
    assert!(is_match);
}

/// The regex is compiled within lazy_static declared at the root module
/// and is used from a mod placed in an external file.
#[bench]
fn lazy_static_backref(b: &mut Bencher) {
    b.iter(|| {
        let is_match = external_mod::lazy_static_backref(TEST_EMAIL);
        test::black_box(is_match);
    });
}

#[test]
fn lazy_static_backref_test() {
    let is_match = external_mod::lazy_static_backref(TEST_EMAIL);
    assert!(is_match);
}

/// The regex is compiled within lazy_static at the module level
/// and is re-initialized within the loop
#[bench]
fn lazy_static_reinit(b: &mut Bencher) {
    b.iter(|| {
        lazy_static::initialize(&COMPILED_REGEX);
        let is_match = COMPILED_REGEX.is_match(TEST_EMAIL);
        test::black_box(is_match);
    });
}

#[test]
fn lazy_static_reinit_test() {
    lazy_static::initialize(&COMPILED_REGEX);
    let is_match = COMPILED_REGEX.is_match(TEST_EMAIL);
    assert!(is_match);
}

mod inner {

    lazy_static! {
        static ref COMPILED_REGEX_INNER: regex::Regex =
            regex::Regex::new(super::LONG_REGEX).unwrap();
    }

    pub(crate) fn lazy_static_local(test_email: &str) -> bool {
        COMPILED_REGEX_INNER.is_match(test_email)
    }
}
