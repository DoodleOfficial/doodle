/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief lib
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
mod config;
mod error_model;
pub mod spell_checker;
mod stupid_backoff;
mod term_freqs;
mod trainer;

pub use config::CorrectionConfig;
pub use error_model::ErrorModel;
pub use spell_checker::Lang;
pub use spell_checker::SpellChecker;
pub use stupid_backoff::StupidBackoff;
pub use term_freqs::TermDict;
pub use trainer::FirstTrainer;
pub use trainer::FirstTrainerResult;
pub use trainer::SecondTrainer;

use fst::Streamer;
use std::ops::Range;

use itertools::intersperse;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("FST error: {0}")]
    Fst(#[from] fst::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Encode error: {0}")]
    Encode(#[from] bincode::error::EncodeError),

    #[error("Decode error: {0}")]
    Decode(#[from] bincode::error::DecodeError),

    #[error("Checker not found")]
    CheckerNotFound,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(
    PartialEq,
    Eq,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
    Clone,
)]
pub struct Correction {
    original: String,
    pub terms: Vec<CorrectionTerm>,
}

#[derive(
    PartialEq,
    Eq,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
    Clone,
)]
pub enum CorrectionTerm {
    Corrected { orig: String, correction: String },
    NotCorrected(String),
}

impl From<Correction> for String {
    fn from(correction: Correction) -> Self {
        intersperse(
            correction.terms.into_iter().map(|term| match term {
                CorrectionTerm::Corrected {
                    orig: _,
                    correction,
                } => correction,
                CorrectionTerm::NotCorrected(orig) => orig,
            }),
            " ".to_string(),
        )
        .collect()
    }
}

impl Correction {
    pub fn empty(original: String) -> Self {
        Self {
            original,
            terms: Vec::new(),
        }
    }

    pub fn push(&mut self, term: CorrectionTerm) {
        self.terms.push(term);
    }

    pub fn is_all_orig(&self) -> bool {
        self.terms
            .iter()
            .all(|term| matches!(term, CorrectionTerm::NotCorrected(_)))
    }
}

pub fn sentence_ranges(text: &str) -> Vec<Range<usize>> {
    let skip = ["mr.", "ms.", "dr."];

    let mut res = Vec::new();
    let mut last_start = 0;

    let text = text.to_ascii_lowercase();

    for (end, _) in text
        .char_indices()
        .filter(|(_, c)| matches!(c, '.' | '\n' | '?' | '!'))
    {
        let end = ceil_char_boundary(&text, end + 1);

        if skip.iter().any(|p| text[last_start..end].ends_with(p)) {
            continue;
        }

        if !text[end..].starts_with(|c: char| c.is_ascii_whitespace()) {
            continue;
        }

        let mut start = last_start;

        while start < end && text[start..].starts_with(|c: char| c.is_whitespace()) {
            start = ceil_char_boundary(&text, start + 1);
        }

        if start > end {
            continue;
        }

        res.push(start..end);

        last_start = end;
    }

    let mut start = last_start;

    while start < text.len() && text[start..].starts_with(|c: char| c.is_whitespace()) {
        start = ceil_char_boundary(&text, start + 1);
    }

    res.push(start..text.len());

    res
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .filter(|s| {
            !s.chars()
                .any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        })
        .map(|s| s.to_string())
        .collect()
}

struct MergePointer<'a> {
    pub(crate) term: String,
    pub(crate) value: u64,
    pub(crate) stream: fst::map::Stream<'a>,
    pub(crate) is_finished: bool,
}

impl MergePointer<'_> {
    pub fn advance(&mut self) -> bool {
        self.is_finished = self
            .stream
            .next()
            .map(|(term, value)| {
                self.term = std::str::from_utf8(term).unwrap().to_string();
                self.value = value;
            })
            .is_none();

        !self.is_finished
    }
}

impl PartialOrd for MergePointer<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MergePointer<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.is_finished, other.is_finished) {
            (true, true) | (false, false) => self.term.cmp(&other.term),
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
        }
    }
}

impl PartialEq for MergePointer<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.term == other.term && self.is_finished == other.is_finished
    }
}

impl Eq for MergePointer<'_> {}

fn ceil_char_boundary(str: &str, index: usize) -> usize {
    let mut res = index;

    while !str.is_char_boundary(res) && res < str.len() {
        res += 1;
    }

    res
}
