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
} // pub struct Pattern

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
