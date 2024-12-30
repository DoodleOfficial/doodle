/**
 * @file ast.rs
 * @author Krisna Pranav
 * @brief ast
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub parser, "/parser.rs");

pub static PARSER: std::sync::LazyLock<parser::BlocksParser> =
    std::sync::LazyLock::new(parser::BlocksParser::new);

#[derive(Debug, PartialEq)]
pub struct RawOptics {
    pub rules: Vec<RawRules>,
    pub host_preferences: Vec<RawOptics>,
    pub discard_non_matching: bool,
}
