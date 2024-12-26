/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief commons
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

pub fn bincode_config() -> bincode::config::Configuration {
    bincode::config::standard()
}
