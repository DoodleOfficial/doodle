/**
 * @file build.rs
 * @author Krisna Pranav
 * @brief build
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

fn main() {
    lalrpop::process_root().unwrap();
}
