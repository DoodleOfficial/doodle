/**
 * @file pattern.rs
 * @author Krisna Pranav
 * @brief robots-policy[pattern]
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Pattern {
    pattern: String,
    len: usize,
}

impl Ord for Pattern {
    fn cmp(&self, other: &Self) -> Ordering {
        self.len().cmp(&other.len()).reverse()
    }
}

impl PartialOrd for Pattern {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Eq for Pattern {}

impl Pattern {
    pub fn new(pattern: &str) -> Self {
        let len = pattern.len();
        let pattern = percent_encode(pattern);
        if pattern.contains('$') {
            return Self {
                pattern: pattern.split('$').next().unwrap().to_string() + "$",
                len,
            };
        }

        Self {
            pattern: pattern.to_string(),
            len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn matches(&self, path: &str) -> bool {
        let path = percent_encode(path);
        let parts = self.pattern.split('*');

        let mut start = 0;

        for (idx, part) in parts.enumerate() {
            if part.ends_with('$') {
                if idx > 0 && part.chars().all(|c| c == '$') {
                    return true;
                }

                let part = part.trim_end_matches('$');

                if idx == 0 {
                    return path == part;
                }

                // rfind because the previous '*' would have matched whatever it could
                match path[start..].rfind(part) {
                    Some(idx) => start += idx + part.len(),
                    _ => {
                        return false;
                    }
                }

                return start == path.len();
            }

            if idx == 0 {
                if !path.starts_with(part) {
                    return false;
                }
                start += part.len();
            } else {
                match path[start..].find(part) {
                    Some(idx) => start += idx + part.len(),
                    None => {
                        return false;
                    }
                }
            }
        }

        true
    }
}

pub(crate) fn percent_encode(input: &str) -> String {
    const FRAGMENT: percent_encoding::AsciiSet = percent_encoding::CONTROLS
        .add(b' ')
        .add(b'"')
        .add(b'<')
        .add(b'>')
        .add(b'`');

    percent_encoding::utf8_percent_encode(
        &percent_encoding::percent_decode_str(input).decode_utf8_lossy(),
        &FRAGMENT,
    )
    .to_string()
}
