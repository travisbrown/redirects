pub mod digest;

use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

const REDIRECT_HTML_PATTERN: &str =
    r#"^<html><body>You are being <a href="([^"]+)">redirected</a>\.</body></html>$"#;
const FILE_PATTERN: &str = r#"^redirects-(.).csv$"#;

pub fn make_redirect_html(url: &str) -> String {
    format!(
        "<html><body>You are being <a href=\"{}\">redirected</a>.</body></html>",
        url
    )
}

pub fn parse_redirect_html(content: &str) -> Option<&str> {
    lazy_static::lazy_static! {
        static ref REDIRECT_HTML_RE: Regex = Regex::new(REDIRECT_HTML_PATTERN).unwrap();
    }

    REDIRECT_HTML_RE
        .captures(content)
        .and_then(|groups| groups.get(1))
        .map(|m| m.as_str())
}

pub fn is_valid_path<P: AsRef<Path>>(path: P) -> bool {
    lazy_static::lazy_static! {
        static ref FILE_RE: Regex = Regex::new(FILE_PATTERN).unwrap();
    }

    path.as_ref()
        .file_name()
        .and_then(|v| v.to_str())
        .and_then(|v| FILE_RE.captures(v))
        .and_then(|groups| groups.get(1))
        .map(|m| FILE_PREFIXES.contains(m.as_str()))
        .unwrap_or(false)
}

pub fn file_prefixes() -> Vec<String> {
    let mut result = FILE_PREFIXES.clone().into_iter().collect::<Vec<_>>();
    result.sort();
    result
}

lazy_static::lazy_static! {
    static ref FILE_PREFIXES: HashSet<String> = {
        let mut prefixes = HashSet::new();
        prefixes.extend(('2'..='7').map(|c| c.to_string()));
        prefixes.extend(('A'..='Z').map(|c| c.to_string()));
        prefixes
    };
}
