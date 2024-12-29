/**
 * @file error_model.rs
 * @author Krisna Pranav
 * @brief error model
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use super::Result;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::Path,
};

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum ErrorType {
    Insertion(char),
    Deletion(char),
    Substitution(char, char),
    Transposition(char, char),
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub struct ErrorSequence(Vec<ErrorType>);

pub fn possible_errors(a: &str, b: &str) -> Option<ErrorSequence> {
    if a == b {
        return None;
    }

    let a_len = a.chars().count();
    let b_len = b.chars().count();
    let mut dp = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        for j in 0..=b_len {
            if i == 0 {
                dp[i][j] = j;
            } else if j == 0 {
                dp[i][j] = i;
            } else {
                let cost = if a.chars().nth(i - 1) == b.chars().nth(j - 1) {
                    0
                } else {
                    1
                };
                dp[i][j] = std::cmp::min(
                    std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                    dp[i - 1][j - 1] + cost,
                );
            }
        }
    }

    let mut i = a_len;
    let mut j = b_len;
    let mut errors = Vec::new();

    while i > 0 && j > 0 {
        let cost = if a.chars().nth(i - 1) == b.chars().nth(j - 1) {
            0
        } else {
            1
        };
        if dp[i][j] == dp[i - 1][j - 1] + cost {
            if cost == 1 {
                errors.push(ErrorType::Substitution(
                    a.chars().nth(i - 1).unwrap(),
                    b.chars().nth(j - 1).unwrap(),
                ));
            }
            i -= 1;
            j -= 1;
        } else if dp[i][j] == dp[i - 1][j] + 1 {
            errors.push(ErrorType::Deletion(a.chars().nth(i - 1).unwrap()));
            i -= 1;
        } else {
            errors.push(ErrorType::Insertion(b.chars().nth(j - 1).unwrap()));
            j -= 1;
        }
    }

    while i > 0 {
        errors.push(ErrorType::Deletion(a.chars().nth(i - 1).unwrap()));
        i -= 1;
    }

    while j > 0 {
        errors.push(ErrorType::Insertion(b.chars().nth(j - 1).unwrap()));
        j -= 1;
    }

    if !errors.is_empty() {
        Some(ErrorSequence(errors))
    } else {
        None
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
struct StoredErrorModel {
    errors: HashMap<String, u64>,
    total: u64,
}

impl From<ErrorModel> for StoredErrorModel {
    fn from(value: ErrorModel) -> Self {
        let stored_errors = value
            .errors
            .into_iter()
            .map(|(error_seq, count)| (serde_json::to_string(&error_seq).unwrap(), count))
            .collect();

        Self {
            errors: stored_errors,
            total: value.total,
        }
    }
}

impl From<StoredErrorModel> for ErrorModel {
    fn from(value: StoredErrorModel) -> Self {
        let errors = value
            .errors
            .into_iter()
            .map(|(error_seq, count)| (serde_json::from_str(&error_seq).unwrap(), count))
            .collect();

        Self {
            errors,
            total: value.total,
        }
    }
}

#[derive(Debug)]
pub struct ErrorModel {
    errors: HashMap<ErrorSequence, u64>,
    total: u64,
}

impl Default for ErrorModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorModel {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            total: 0,
        }
    }

    pub fn save<P: AsRef<Path>>(self, path: P) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let wrt = BufWriter::new(file);

        serde_json::to_writer_pretty(wrt, &StoredErrorModel::from(self))?;

        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new().read(true).open(path)?;

        let rdr = BufReader::new(file);

        let stored: StoredErrorModel = serde_json::from_reader(rdr)?;

        Ok(stored.into())
    }

    pub fn add(&mut self, a: &str, b: &str) {
        if let Some(errors) = possible_errors(a, b) {
            *self.errors.entry(errors).or_insert(0) += 1;
            self.total += 1;
        }
    }

    pub fn prob(&self, error: &ErrorSequence) -> f64 {
        let count = self.errors.get(error).unwrap_or(&0);
        *count as f64 / self.total as f64
    }

    pub fn log_prob(&self, error: &ErrorSequence) -> f64 {
        match self.errors.get(error) {
            Some(count) => (*count as f64).log2() - ((self.total + 1) as f64).log2(),
            None => 0.0 - ((self.total + 1) as f64).log2(),
        }
    }
}
