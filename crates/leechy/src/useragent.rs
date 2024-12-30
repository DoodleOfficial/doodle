/**
 * @file useragent.rs
 * @author Krisna Pranav
 * @brief user agent
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use rand::prelude::*;
use std::sync::LazyLock;

static USER_AGENTS: LazyLock<Vec<(usize, String)>> = LazyLock::new(|| {
    include_str!("useragents.txt")
        .lines()
        .map(|line| line.trim().to_string())
        .enumerate()
        .collect()
});

pub struct UserAgent(String);

impl UserAgent {
    pub fn random_weighted() -> Self {
        let mut rng = thread_rng();

        UserAgent(
            USER_AGENTS
                .choose_weighted(&mut rng, |(rank, _)| 1 / (rank + 1))
                .unwrap()
                .1
                .clone(),
        )
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for UserAgent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
