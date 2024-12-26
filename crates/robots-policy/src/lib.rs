/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief robots-policy.
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

const MAX_CHAR_LIMIT_DEFAULT: usize = 512_000;

mod parser;
mod pattern;

use crate::parser::Line;
use itertools::Itertools;
use pattern::Pattern;
use std::time::Duration;
use url::Url;

#[derive(Debug, PartialEq, Eq)]
enum Directive {
    Allow,
    Disallow,
}

impl Ord for Directive {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Allow, Self::Disallow) => std::cmp::Ordering::Less,
            (Self::Disallow, Self::Allow) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    }
} // impl Ord for Directive

impl PartialOrd for Directive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
} // impl PartialOrd for Directive

#[derive(Debug, PartialEq, Eq)]
struct Rule {
    pattern: Pattern,
    directive: Directive,
} // struct Rule

impl Ord for Rule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pattern
            .cmp(&other.pattern)
            .then(self.directive.cmp(&other.directive))
    }
}
