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

use itertools::Itertools;
use std::time::Duration;
use url::Url;

use crate::parser::Line;
