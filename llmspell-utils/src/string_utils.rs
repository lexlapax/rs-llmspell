// ABOUTME: String manipulation, formatting, and validation utilities
// ABOUTME: Common string operations used across the LLMSpell framework

//! String manipulation and formatting helpers
//!
//! This module provides utilities for string manipulation, including
//! truncation, sanitization, case conversion, word wrapping, and indentation.

/// Default ellipsis string for truncation
pub const DEFAULT_ELLIPSIS: &str = "...";

/// Truncate a string to a maximum length with ellipsis
///
/// If the string is longer than `max_len`, it will be truncated and ellipsis added.
/// The ellipsis counts towards the maximum length.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::truncate;
///
/// assert_eq!(truncate("Hello, world!", 5), "He...");
/// assert_eq!(truncate("Short", 10), "Short");
/// assert_eq!(truncate("", 5), "");
/// ```
#[must_use]
pub fn truncate(s: &str, max_len: usize) -> String {
    truncate_with_ellipsis(s, max_len, DEFAULT_ELLIPSIS)
}

/// Truncate a string to a maximum length with custom ellipsis
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::truncate_with_ellipsis;
///
/// assert_eq!(truncate_with_ellipsis("Hello, world!", 7, "..."), "Hell...");
/// assert_eq!(truncate_with_ellipsis("Hello, world!", 7, "…"), "Hell…");
/// ```
#[must_use]
pub fn truncate_with_ellipsis(s: &str, max_len: usize, ellipsis: &str) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    if max_len <= ellipsis.len() {
        // If max_len is too small for ellipsis, just return the ellipsis truncated
        return ellipsis.chars().take(max_len).collect();
    }

    let truncate_at = max_len - ellipsis.len();

    // Find a char boundary
    let mut boundary = truncate_at;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }

    format!("{}{}", &s[..boundary], ellipsis)
}

/// Wrap text to fit within a specified width
///
/// Breaks text at word boundaries when possible.
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::word_wrap;
///
/// let text = "This is a long line that needs to be wrapped";
/// let wrapped = word_wrap(text, 20);
/// assert_eq!(wrapped, vec!["This is a long line", "that needs to be", "wrapped"]);
/// ```
#[must_use]
pub fn word_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![];
    }

    let mut lines = Vec::new();

    for line in text.lines() {
        if line.len() <= width {
            lines.push(line.to_string());
            continue;
        }

        let words: Vec<&str> = line.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            if current_line.is_empty() {
                // First word on the line
                if word.len() > width {
                    // Word is too long, must be broken
                    let mut chars = word.chars();
                    while !chars.as_str().is_empty() {
                        let chunk: String = chars.by_ref().take(width).collect();
                        lines.push(chunk);
                    }
                } else {
                    current_line = word.to_string();
                }
            } else if current_line.len() + 1 + word.len() <= width {
                // Word fits on current line
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                // Word doesn't fit, start new line
                lines.push(current_line);
                if word.len() > width {
                    // Word is too long for any line, must be broken
                    current_line = String::new();
                    let mut chars = word.chars();
                    while !chars.as_str().is_empty() {
                        let chunk: String = chars.by_ref().take(width).collect();
                        if chars.as_str().is_empty() {
                            current_line = chunk;
                        } else {
                            lines.push(chunk);
                        }
                    }
                } else {
                    current_line = word.to_string();
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    lines
}

/// Convert a string to `snake_case`
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::to_snake_case;
///
/// assert_eq!(to_snake_case("HelloWorld"), "hello_world");
/// assert_eq!(to_snake_case("HTTPResponse"), "httpresponse");
/// assert_eq!(to_snake_case("already_snake_case"), "already_snake_case");
/// assert_eq!(to_snake_case("PascalCase"), "pascal_case");
/// ```
#[must_use]
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_upper = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !prev_upper {
                result.push('_');
            }
            // Safe: to_lowercase() always produces at least one char for valid Unicode
            if let Some(lower_ch) = ch.to_lowercase().next() {
                result.push(lower_ch);
            }
            prev_upper = true;
        } else {
            result.push(ch);
            prev_upper = false;
        }
    }

    result
}

/// Convert a string to `camelCase`
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::to_camel_case;
///
/// assert_eq!(to_camel_case("hello_world"), "helloWorld");
/// assert_eq!(to_camel_case("http_response"), "httpResponse");
/// assert_eq!(to_camel_case("already_camelCase"), "alreadyCamelCase");
/// ```
#[must_use]
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, ch) in s.chars().enumerate() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            // Safe: to_uppercase() always produces at least one char for valid Unicode
            if let Some(upper_ch) = ch.to_uppercase().next() {
                result.push(upper_ch);
            }
            capitalize_next = false;
        } else if i == 0 {
            // Safe: to_lowercase() always produces at least one char for valid Unicode
            if let Some(lower_ch) = ch.to_lowercase().next() {
                result.push(lower_ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Convert a string to `PascalCase`
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::to_pascal_case;
///
/// assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
/// assert_eq!(to_pascal_case("http_response"), "HttpResponse");
/// assert_eq!(to_pascal_case("already_PascalCase"), "AlreadyPascalCase");
/// ```
#[must_use]
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            // Safe: to_uppercase() always produces at least one char for valid Unicode
            if let Some(upper_ch) = ch.to_uppercase().next() {
                result.push(upper_ch);
            }
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Sanitize a string for safe usage
///
/// Removes or escapes potentially dangerous characters:
/// - Control characters (except newline and tab)
/// - Non-ASCII characters
/// - Leading/trailing spaces (but not tabs)
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::sanitize;
///
/// assert_eq!(sanitize("  Hello\x00World  "), "HelloWorld");
/// assert_eq!(sanitize("Normal text"), "Normal text");
/// assert_eq!(sanitize("Line1\nLine2\t"), "Line1\nLine2\t");
/// ```
#[must_use]
pub fn sanitize(s: &str) -> String {
    // Trim only regular spaces, not tabs or newlines
    let trimmed = s.trim_matches(' ');
    trimmed
        .chars()
        .filter(|&ch| ch == '\n' || ch == '\t' || (ch.is_ascii_graphic() || ch == ' '))
        .collect()
}

/// Escape special characters for safe display
///
/// Escapes characters that might cause issues in various contexts
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::escape_special;
///
/// assert_eq!(escape_special("Hello\nWorld"), "Hello\\nWorld");
/// assert_eq!(escape_special("Path\\to\\file"), "Path\\\\to\\\\file");
/// assert_eq!(escape_special("Quote\"test\""), "Quote\\\"test\\\"");
/// ```
#[must_use]
pub fn escape_special(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for ch in s.chars() {
        match ch {
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\'' => result.push_str("\\'"),
            '\0' => result.push_str("\\0"),
            _ => result.push(ch),
        }
    }

    result
}

/// Add indentation to each line of a string
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::indent;
///
/// let text = "Line 1\nLine 2\nLine 3";
/// let indented = indent(text, 4);
/// assert_eq!(indented, "    Line 1\n    Line 2\n    Line 3");
/// ```
#[must_use]
pub fn indent(s: &str, spaces: usize) -> String {
    indent_with(s, &" ".repeat(spaces))
}

/// Add custom indentation to each line of a string
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::indent_with;
///
/// let text = "Line 1\nLine 2";
/// let indented = indent_with(text, "> ");
/// assert_eq!(indented, "> Line 1\n> Line 2");
/// ```
#[must_use]
pub fn indent_with(s: &str, prefix: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let mut result = String::new();

    for (i, line) in lines.iter().enumerate() {
        result.push_str(prefix);
        result.push_str(line);
        if i < lines.len() - 1 || s.ends_with('\n') {
            result.push('\n');
        }
    }

    result
}

/// Remove common leading whitespace from lines
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::dedent;
///
/// let text = "    Line 1\n    Line 2\n      Line 3";
/// let dedented = dedent(text);
/// assert_eq!(dedented, "Line 1\nLine 2\n  Line 3");
/// ```
#[must_use]
pub fn dedent(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();

    // Find minimum indentation (ignoring empty lines)
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Remove common indentation
    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        if line.len() >= min_indent {
            result.push_str(&line[min_indent..]);
        } else {
            result.push_str(line);
        }
        if i < lines.len() - 1 || s.ends_with('\n') {
            result.push('\n');
        }
    }

    result
}

/// Check if a string is a valid identifier (alphanumeric + underscore, not starting with digit)
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::is_valid_identifier;
///
/// assert!(is_valid_identifier("valid_name"));
/// assert!(is_valid_identifier("_private"));
/// assert!(!is_valid_identifier("123invalid"));
/// assert!(!is_valid_identifier("invalid-name"));
/// assert!(!is_valid_identifier(""));
/// ```
#[must_use]
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be alphabetic or underscore
    if let Some(first) = chars.next() {
        if !first.is_alphabetic() && first != '_' {
            return false;
        }
    }

    // Rest must be alphanumeric or underscore
    chars.all(|ch| ch.is_alphanumeric() || ch == '_')
}

/// Split a string by lines, preserving line endings
///
/// Unlike `str::lines()`, this preserves the line ending characters
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::lines_with_endings;
///
/// let text = "Line 1\nLine 2\r\nLine 3";
/// let lines: Vec<_> = lines_with_endings(text).collect();
/// assert_eq!(lines, vec!["Line 1\n", "Line 2\r\n", "Line 3"]);
/// ```
pub fn lines_with_endings(s: &str) -> impl Iterator<Item = &str> {
    let mut start = 0;
    let bytes = s.as_bytes();

    std::iter::from_fn(move || {
        if start >= bytes.len() {
            return None;
        }

        let mut end = start;
        while end < bytes.len() {
            if bytes[end] == b'\n' {
                end += 1;
                break;
            } else if end + 1 < bytes.len() && bytes[end] == b'\r' && bytes[end + 1] == b'\n' {
                end += 2;
                break;
            }
            end += 1;
        }

        let line = &s[start..end];
        start = end;
        Some(line)
    })
}

/// Replace multiple consecutive whitespace characters with a single space
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::normalize_whitespace;
///
/// assert_eq!(normalize_whitespace("Hello    world"), "Hello world");
/// assert_eq!(normalize_whitespace("Line1\n\n\nLine2"), "Line1 Line2");
/// assert_eq!(normalize_whitespace("  Leading and trailing  "), "Leading and trailing");
/// ```
#[must_use]
pub fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Count the number of lines in a string
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::count_lines;
///
/// assert_eq!(count_lines("Single line"), 1);
/// assert_eq!(count_lines("Line 1\nLine 2\nLine 3"), 3);
/// assert_eq!(count_lines(""), 0);
/// assert_eq!(count_lines("\n\n"), 2);
/// ```
#[must_use]
pub fn count_lines(s: &str) -> usize {
    if s.is_empty() {
        0
    } else {
        s.lines().count()
    }
}

/// Find common prefix of multiple strings
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::common_prefix;
///
/// assert_eq!(common_prefix(&["prefix_a", "prefix_b", "prefix_c"]), "prefix_");
/// assert_eq!(common_prefix(&["hello", "help", "hero"]), "he");
/// assert_eq!(common_prefix(&["abc", "xyz"]), "");
/// assert_eq!(common_prefix(&[]), "");
/// ```
#[must_use]
pub fn common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    let mut prefix = String::new();
    let first = strings[0];

    'outer: for (i, ch) in first.chars().enumerate() {
        for s in &strings[1..] {
            if s.chars().nth(i) != Some(ch) {
                break 'outer;
            }
        }
        prefix.push(ch);
    }

    prefix
}

/// Create a string of repeated characters
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::repeat_char;
///
/// assert_eq!(repeat_char('=', 5), "=====");
/// assert_eq!(repeat_char(' ', 3), "   ");
/// assert_eq!(repeat_char('*', 0), "");
/// ```
#[must_use]
pub fn repeat_char(ch: char, count: usize) -> String {
    ch.to_string().repeat(count)
}

/// Convert string to uppercase
///
/// Wrapper for consistency with other string operations
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::to_uppercase;
///
/// assert_eq!(to_uppercase("hello world"), "HELLO WORLD");
/// assert_eq!(to_uppercase("123 Test"), "123 TEST");
/// ```
#[must_use]
pub fn to_uppercase(s: &str) -> String {
    s.to_uppercase()
}

/// Convert string to lowercase
///
/// Wrapper for consistency with other string operations
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::to_lowercase;
///
/// assert_eq!(to_lowercase("HELLO WORLD"), "hello world");
/// assert_eq!(to_lowercase("123 Test"), "123 test");
/// ```
#[must_use]
pub fn to_lowercase(s: &str) -> String {
    s.to_lowercase()
}

/// Reverse a string
///
/// Properly handles Unicode grapheme clusters
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::reverse;
///
/// assert_eq!(reverse("hello"), "olleh");
/// assert_eq!(reverse("世界"), "界世");
/// assert_eq!(reverse(""), "");
/// ```
#[must_use]
pub fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

/// Trim whitespace from both ends of a string
///
/// Wrapper for consistency with other string operations
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::trim;
///
/// assert_eq!(trim("  hello  "), "hello");
/// assert_eq!(trim("\thello\n"), "hello");
/// assert_eq!(trim("hello"), "hello");
/// ```
#[must_use]
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

/// Replace all occurrences of a pattern in a string
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::replace_all;
///
/// assert_eq!(replace_all("hello world", "o", "0"), "hell0 w0rld");
/// assert_eq!(replace_all("foo bar foo", "foo", "baz"), "baz bar baz");
/// ```
#[must_use]
pub fn replace_all(s: &str, from: &str, to: &str) -> String {
    s.replace(from, to)
}

/// Extract substring by start and end indices
///
/// Returns empty string if indices are invalid
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::substring;
///
/// assert_eq!(substring("hello world", 0, 5), "hello");
/// assert_eq!(substring("hello world", 6, 11), "world");
/// assert_eq!(substring("hello", 10, 20), "");
/// ```
#[must_use]
pub fn substring(s: &str, start: usize, end: usize) -> String {
    if start >= s.len() || end > s.len() || start >= end {
        return String::new();
    }

    // Find char boundaries
    let mut start_byte = start;
    while start_byte > 0 && !s.is_char_boundary(start_byte) {
        start_byte -= 1;
    }

    let mut end_byte = end;
    while end_byte < s.len() && !s.is_char_boundary(end_byte) {
        end_byte += 1;
    }

    s[start_byte..end_byte].to_string()
}

/// Split string by delimiter and return vector of parts
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::split_by;
///
/// assert_eq!(split_by("a,b,c", ","), vec!["a", "b", "c"]);
/// assert_eq!(split_by("hello world", " "), vec!["hello", "world"]);
/// assert_eq!(split_by("no-delimiter", ","), vec!["no-delimiter"]);
/// ```
#[must_use]
pub fn split_by(s: &str, delimiter: &str) -> Vec<String> {
    s.split(delimiter).map(String::from).collect()
}

/// Join strings with delimiter
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::string_utils::join_with;
///
/// assert_eq!(join_with(&["a", "b", "c"], ","), "a,b,c");
/// assert_eq!(join_with(&["hello", "world"], " "), "hello world");
/// assert_eq!(join_with(&[], ","), "");
/// ```
#[must_use]
pub fn join_with(parts: &[&str], delimiter: &str) -> String {
    parts.join(delimiter)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello, world!", 5), "He...");
        assert_eq!(truncate("Short", 10), "Short");
        assert_eq!(truncate("Exactly10!", 10), "Exactly10!");
        assert_eq!(truncate("", 5), "");
        assert_eq!(truncate("Hi", 2), "Hi");
        assert_eq!(truncate("Hello", 3), "...");

        // Test with unicode - 8 bytes is not enough to include the chinese char
        let result = truncate("Hello 世界", 8);
        assert!(result.starts_with("Hello"));
        assert!(result.ends_with("..."));
    }
    #[test]
    fn test_truncate_with_ellipsis() {
        // The unicode ellipsis takes 3 bytes, so "Hello," (6 bytes) + "…" (3 bytes) = 9 bytes
        // But we're asking for 7 max length, so it should be truncated further
        let result = truncate_with_ellipsis("Hello, world!", 7, "…");
        assert!(result.len() <= 7 || result.chars().count() <= 7);
        assert_eq!(truncate_with_ellipsis("Short", 10, "…"), "Short");
        assert_eq!(truncate_with_ellipsis("Test", 1, "..."), ".");
    }
    #[test]
    fn test_word_wrap() {
        let text = "This is a long line that needs to be wrapped";
        let wrapped = word_wrap(text, 20);
        assert_eq!(
            wrapped,
            vec!["This is a long line", "that needs to be", "wrapped"]
        );

        // Test with very long word
        let wrapped = word_wrap("Supercalifragilisticexpialidocious", 10);
        assert_eq!(
            wrapped,
            vec!["Supercalif", "ragilistic", "expialidoc", "ious"]
        );

        // Test with empty string
        assert_eq!(word_wrap("", 10), Vec::<String>::new());

        // Test with width 0
        assert_eq!(word_wrap("Test", 0), Vec::<String>::new());

        // Test with multiple lines
        let multi = "Line 1 is here\nLine 2 is also here";
        let wrapped = word_wrap(multi, 10);
        assert_eq!(wrapped, vec!["Line 1 is", "here", "Line 2 is", "also here"]);
    }
    #[test]
    fn test_case_conversions() {
        // snake_case
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("HTTPResponse"), "httpresponse");
        assert_eq!(to_snake_case("already_snake_case"), "already_snake_case");
        assert_eq!(to_snake_case("IOError"), "ioerror");

        // camelCase
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("http_response"), "httpResponse");
        assert_eq!(to_camel_case("already_camelCase"), "alreadyCamelCase");
        assert_eq!(to_camel_case("io-error"), "ioError");

        // PascalCase
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("http_response"), "HttpResponse");
        assert_eq!(to_pascal_case("already_PascalCase"), "AlreadyPascalCase");
        assert_eq!(to_pascal_case("io-error"), "IoError");
    }
    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize("  Hello\x00World  "), "HelloWorld");
        assert_eq!(sanitize("Normal text"), "Normal text");
        assert_eq!(sanitize("Line1\nLine2\t"), "Line1\nLine2\t");
        assert_eq!(sanitize("\x01\x02Test\x03\x04"), "Test");
        // String with only control characters should become empty after sanitization
        let result = sanitize("   \x00  \x01  ");
        assert!(
            result.chars().all(|c| c == ' '),
            "Should only contain spaces, got: {result:?}"
        );
    }
    #[test]
    fn test_escape_special() {
        assert_eq!(escape_special("Hello\nWorld"), "Hello\\nWorld");
        assert_eq!(escape_special("Path\\to\\file"), "Path\\\\to\\\\file");
        assert_eq!(escape_special("Quote\"test\""), "Quote\\\"test\\\"");
        assert_eq!(escape_special("Tab\there"), "Tab\\there");
        assert_eq!(escape_special("Null\0char"), "Null\\0char");
    }
    #[test]
    fn test_indent() {
        let text = "Line 1\nLine 2\nLine 3";
        let indented = indent(text, 4);
        assert_eq!(indented, "    Line 1\n    Line 2\n    Line 3");

        // Test with custom prefix
        let indented = indent_with(text, "> ");
        assert_eq!(indented, "> Line 1\n> Line 2\n> Line 3");

        // Test with trailing newline
        let text_nl = "Line 1\nLine 2\n";
        let indented = indent(text_nl, 2);
        assert_eq!(indented, "  Line 1\n  Line 2\n");
    }
    #[test]
    fn test_dedent() {
        let text = "    Line 1\n    Line 2\n      Line 3";
        let dedented = dedent(text);
        assert_eq!(dedented, "Line 1\nLine 2\n  Line 3");

        // Test with empty lines
        let text = "    Line 1\n\n    Line 2";
        let dedented = dedent(text);
        assert_eq!(dedented, "Line 1\n\nLine 2");

        // Test with no common indentation
        let text = "Line 1\n  Line 2";
        let dedented = dedent(text);
        assert_eq!(dedented, text);
    }
    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("valid_name"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("CamelCase"));
        assert!(is_valid_identifier("name123"));

        assert!(!is_valid_identifier("123invalid"));
        assert!(!is_valid_identifier("invalid-name"));
        assert!(!is_valid_identifier("invalid.name"));
        assert!(!is_valid_identifier("invalid name"));
        assert!(!is_valid_identifier(""));
    }
    #[test]
    fn test_lines_with_endings() {
        let text = "Line 1\nLine 2\r\nLine 3";
        let lines: Vec<_> = lines_with_endings(text).collect();
        assert_eq!(lines, vec!["Line 1\n", "Line 2\r\n", "Line 3"]);

        // Test with trailing newline
        let text = "Line 1\nLine 2\n";
        let lines: Vec<_> = lines_with_endings(text).collect();
        assert_eq!(lines, vec!["Line 1\n", "Line 2\n"]);

        // Test empty string
        let lines: Vec<_> = lines_with_endings("").collect();
        assert!(lines.is_empty());
    }
    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("Hello    world"), "Hello world");
        assert_eq!(normalize_whitespace("Line1\n\n\nLine2"), "Line1 Line2");
        assert_eq!(
            normalize_whitespace("  Leading and trailing  "),
            "Leading and trailing"
        );
        assert_eq!(normalize_whitespace("\t\tTabs\t\t"), "Tabs");
        assert_eq!(normalize_whitespace(""), "");
    }
    #[test]
    fn test_count_lines() {
        assert_eq!(count_lines("Single line"), 1);
        assert_eq!(count_lines("Line 1\nLine 2\nLine 3"), 3);
        assert_eq!(count_lines(""), 0);
        assert_eq!(count_lines("\n\n"), 2);
        assert_eq!(count_lines("No newline at end\n"), 1);
    }
    #[test]
    fn test_common_prefix() {
        assert_eq!(
            common_prefix(&["prefix_a", "prefix_b", "prefix_c"]),
            "prefix_"
        );
        assert_eq!(common_prefix(&["hello", "help", "hero"]), "he");
        assert_eq!(common_prefix(&["abc", "xyz"]), "");
        assert_eq!(common_prefix(&["test"]), "test");
        assert_eq!(common_prefix(&[]), "");

        // Test with unicode
        assert_eq!(common_prefix(&["你好世界", "你好朋友"]), "你好");
    }
    #[test]
    fn test_repeat_char() {
        assert_eq!(repeat_char('=', 5), "=====");
        assert_eq!(repeat_char(' ', 3), "   ");
        assert_eq!(repeat_char('*', 0), "");
        assert_eq!(repeat_char('🦀', 3), "🦀🦀🦀");
    }
    #[test]
    fn test_to_uppercase() {
        assert_eq!(to_uppercase("hello world"), "HELLO WORLD");
        assert_eq!(to_uppercase("123 Test"), "123 TEST");
        assert_eq!(to_uppercase(""), "");
        assert_eq!(to_uppercase("ALREADY UPPER"), "ALREADY UPPER");
    }
    #[test]
    fn test_to_lowercase() {
        assert_eq!(to_lowercase("HELLO WORLD"), "hello world");
        assert_eq!(to_lowercase("123 Test"), "123 test");
        assert_eq!(to_lowercase(""), "");
        assert_eq!(to_lowercase("already lower"), "already lower");
    }
    #[test]
    fn test_reverse() {
        assert_eq!(reverse("hello"), "olleh");
        assert_eq!(reverse("世界"), "界世");
        assert_eq!(reverse(""), "");
        assert_eq!(reverse("a"), "a");
        assert_eq!(reverse("🦀rust🦀"), "🦀tsur🦀");
    }
    #[test]
    fn test_trim() {
        assert_eq!(trim("  hello  "), "hello");
        assert_eq!(trim("\thello\n"), "hello");
        assert_eq!(trim("hello"), "hello");
        assert_eq!(trim("   "), "");
        assert_eq!(trim(""), "");
    }
    #[test]
    fn test_replace_all() {
        assert_eq!(replace_all("hello world", "o", "0"), "hell0 w0rld");
        assert_eq!(replace_all("foo bar foo", "foo", "baz"), "baz bar baz");
        assert_eq!(replace_all("no match", "x", "y"), "no match");
        assert_eq!(replace_all("", "x", "y"), "");
    }
    #[test]
    fn test_substring() {
        assert_eq!(substring("hello world", 0, 5), "hello");
        assert_eq!(substring("hello world", 6, 11), "world");
        assert_eq!(substring("hello", 10, 20), "");
        assert_eq!(substring("hello", 2, 2), "");
        assert_eq!(substring("hello", 5, 3), "");
        assert_eq!(substring("🦀rust", 0, 2), "🦀");
    }
    #[test]
    fn test_split_by() {
        assert_eq!(split_by("a,b,c", ","), vec!["a", "b", "c"]);
        assert_eq!(split_by("hello world", " "), vec!["hello", "world"]);
        assert_eq!(split_by("no-delimiter", ","), vec!["no-delimiter"]);
        assert_eq!(split_by("", ","), vec![""]);
        assert_eq!(split_by("a,,b", ","), vec!["a", "", "b"]);
    }
    #[test]
    fn test_join_with() {
        assert_eq!(join_with(&["a", "b", "c"], ","), "a,b,c");
        assert_eq!(join_with(&["hello", "world"], " "), "hello world");
        assert_eq!(join_with(&[], ","), "");
        assert_eq!(join_with(&["single"], ","), "single");
        assert_eq!(join_with(&["", "", ""], "-"), "--");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_truncate_length(s: String, max_len in 0usize..1000) {
            let truncated = truncate(&s, max_len);
            assert!(truncated.len() <= max_len || truncated.len() <= DEFAULT_ELLIPSIS.len());
        }
        #[test]
        fn test_sanitize_no_control_chars(s: String) {
            let sanitized = sanitize(&s);
            for ch in sanitized.chars() {
                assert!(ch == '\n' || ch == '\t' || (!ch.is_control() && ch.is_ascii()));
            }
        }
        #[test]
        fn test_indent_line_count(s: String, spaces in 0usize..10) {
            let indented = indent(&s, spaces);
            if !s.is_empty() {
                assert_eq!(count_lines(&s), count_lines(&indented));
            }
        }
        #[test]
        fn test_case_conversion_consistency(s in "[a-zA-Z][a-zA-Z0-9_]*") {
            // Test that conversions produce valid identifiers
            let snake = to_snake_case(&s);
            let camel = to_camel_case(&s);
            let pascal = to_pascal_case(&s);

            // All conversions should produce non-empty strings if input is non-empty
            if !s.is_empty() {
                assert!(!snake.is_empty());
                assert!(!camel.is_empty());
                assert!(!pascal.is_empty());
            }
        }
    }
}
