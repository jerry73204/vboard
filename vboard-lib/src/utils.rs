pub fn percent_encode(text: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    utf8_percent_encode(text, NON_ALPHANUMERIC).to_string()
}
