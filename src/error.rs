use std::fmt;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

/// True for a character that must not reach a terminal or split a TSV line: controls (NUL, ESC, tab,
/// line feed, C1) and invisible or direction-changing characters.
pub(crate) fn is_unsafe(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{ad}'
                | '\u{61c}'
                | '\u{180e}'
                | '\u{200b}'..='\u{200f}'
                | '\u{2028}'..='\u{202e}'
                | '\u{2060}'..='\u{206f}'
                | '\u{feff}'
                | '\u{fff9}'..='\u{fffb}'
                | '\u{e0000}'..='\u{e007f}'
        )
}

/// Control and invisible characters become `?`.
pub(crate) fn clean(s: &str) -> String {
    s.chars()
        .map(|c| if is_unsafe(c) { '?' } else { c })
        .collect()
}

impl Error {
    /// Every message is cleaned here, so text from input files and paths never reaches stderr raw.
    pub fn msg(m: impl Into<String>) -> Self {
        Self(clean(&m.into()))
    }
    pub fn message(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
