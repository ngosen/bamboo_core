//! Provides functions and tables for encoding Vietnamese Unicode text into legacy character sets.
//!
//! Supported character sets include:
//! - **Unicode** (precomposed NFC)
//! - **Unicode tổ hợp** (decomposed NFD)
//! - **TCVN3 (ABC)**
//! - **VNI Windows**
//! - **Windows 1258 codepage**
//! - **VIQR**
//! - **VISCII**
//! - **VPS**
//! - **BKHCM 1** / **BKHCM 2**
//! - **Vietware X** / **Vietware Full**
//! - **UTF-8**
//! - **NCR Decimal** / **NCR Hex**
//! - **Unicode C string Hex** / **Unicode C string Decimal**

use crate::charset_def::{get_charset_definition, get_charset_definitions};

static UNICODE: &str = "Unicode";

/// Encodes a Vietnamese Unicode string into a specific character set.
///
/// If `charset_name` is `"Unicode"` or the input is empty, returns a clone of `input`.
/// Unrecognized characters or unknown charset names pass through unchanged.
///
/// # Arguments
/// * `charset_name` - Name of the target character set (e.g., `"TCVN3 (ABC)"`, `"VNI Windows"`, `"VIQR"`).
/// * `input` - The source Unicode string.
///
/// # Example
/// ```rust
/// use bamboo_core::advanced::encode;
///
/// let viqr = encode("VIQR", "tiếng Việt");
/// assert_eq!(viqr, "tie^'ng Vie^.t");
/// ```
pub fn encode(charset_name: &str, input: &str) -> String {
    if charset_name == UNICODE || input.is_empty() {
        return input.to_string();
    }

    match get_charset_definition(charset_name) {
        Some(charset_def) => {
            let mut output = String::with_capacity(input.len());
            for char in input.chars() {
                if char.is_ascii() {
                    output.push(char);
                } else {
                    match charset_def.get(&char) {
                        Some(encoded) => output.push_str(encoded),
                        None => output.push(char),
                    }
                }
            }
            output
        }
        None => input.to_string(),
    }
}

/// Returns an iterator over all supported character set names without heap allocations.
///
/// # Example
/// ```rust
/// use bamboo_core::advanced::charset_names;
///
/// let names: Vec<&'static str> = charset_names().collect();
/// assert!(names.contains(&"Unicode"));
/// assert!(names.contains(&"TCVN3 (ABC)"));
/// assert!(names.contains(&"VIQR"));
/// ```
pub fn charset_names() -> impl Iterator<Item = &'static str> {
    std::iter::once(UNICODE).chain(get_charset_definitions().keys().copied())
}

/// Returns a newly allocated list of all supported character set names.
pub fn get_charset_name() -> Vec<String> {
    charset_names().map(String::from).collect()
}

/// Alias for [`get_charset_name`].
pub fn get_charset_names() -> Vec<String> {
    get_charset_name()
}
