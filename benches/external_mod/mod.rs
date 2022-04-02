lazy_static::lazy_static! {
    static ref COMPILED_REGEX_OUTER: regex::Regex = regex::Regex::new(super::LONG_REGEX).unwrap();
}

pub(crate) fn lazy_static_external(test_email: &str) -> bool {
    COMPILED_REGEX_OUTER.is_match(test_email)
}

pub(crate) fn lazy_static_backref(test_email: &str) -> bool {
    super::COMPILED_REGEX.is_match(test_email)
}