/**
 * @file config.rs
 * @author Krisna Pranav
 * @brief config
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

fn misspelled_prob() -> f64 {
    0.1
}

fn correction_threshold() -> f64 {
    50.0
}

fn lm_prob_weight() -> f64 {
    5.77
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct CorrectionConfig {
    #[serde(default = "misspelled_prob")]
    pub misspelled_prob: f64,

    #[serde(default = "lm_prob_weight")]
    pub lm_prob_weight: f64,

    #[serde(default = "correction_threshold")]
    pub correction_threshold: f64,
}

impl Default for CorrectionConfig {
    fn default() -> Self {
        Self {
            misspelled_prob: misspelled_prob(),
            lm_prob_weight: lm_prob_weight(),
            correction_threshold: correction_threshold(),
        }
    }
}

pub fn bincode_config() -> bincode::config::Configuration {
    bincode::config::standard()
}
