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
use super::lexer;
use super::Error;
use super::Result as ModResult;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub parser, "/parser.rs");

pub static PARSER: std::sync::LazyLock<parser::BlocksParser> =
    std::sync::LazyLock::new(parser::BlocksParser::new);

#[derive(Debug, PartialEq)]
pub struct RawOptic {
    pub rules: Vec<RawRule>,
    pub host_preferences: Vec<RawHostPreference>,
    pub discard_non_matching: bool,
}

impl From<Vec<RawOpticBlock>> for RawOptic {
    fn from(blocks: Vec<RawOpticBlock>) -> Self {
        let mut rules = Vec::new();
        let mut host_preferences = Vec::new();
        let mut discard_non_matching = false;

        for block in blocks {
            match block {
                RawOpticBlock::Rule(rule) => rules.push(rule),
                RawOpticBlock::HostPreference(pref) => host_preferences.push(pref),
                RawOpticBlock::DiscardNonMatching => discard_non_matching = true,
            }
        }

        RawOptic {
            rules,
            host_preferences,
            discard_non_matching,
        }
    }
}

#[derive(Debug)]
pub enum RawOpticBlock {
    Rule(RawRule),
    HostPreference(RawHostPreference),
    DiscardNonMatching,
}

#[derive(Debug, PartialEq)]
pub struct RawRule {
    pub matches: Vec<RawMatchBlock>,
    pub action: Option<RawAction>,
}

#[derive(Debug, PartialEq)]
pub enum RawHostPreference {
    Like(String),
    Dislike(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct RawMatchBlock(pub Vec<RawMatchPart>);

#[derive(Debug, PartialEq, Clone)]
pub enum RawMatchPart {
    Site(String),
    Url(String),
    Domain(String),
    Title(String),
    Description(String),
    Content(String),
    MicroformatTag(String),
    Schema(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum RawAction {
    Boost(u64),
    Downrank(u64),
    Discard,
}

pub fn parse(optic: &str) -> ModResult<RawOptic> {
    match PARSER.parse(lexer::lex(optic)) {
        Ok(blocks) => Ok(RawOptic::from(blocks)),
        Err(error) => match error {
            lalrpop_util::ParseError::InvalidToken { location: _ } => unreachable!(
                "this is a lexing error, which should be caught earlier since we use logos"
            ),
            lalrpop_util::ParseError::UnrecognizedEof {
                location: _,
                expected,
            } => Err(Error::UnexpectedEof { expected }),
            lalrpop_util::ParseError::UnrecognizedToken {
                token: (start, tok, end),
                expected,
            } => Err(Error::UnexpectedToken {
                token: (start, tok.to_string(), end),
                expected,
            }),
            lalrpop_util::ParseError::ExtraToken {
                token: (start, tok, end),
            } => Err(Error::UnrecognizedToken {
                token: (start, tok.to_string(), end),
            }),
            lalrpop_util::ParseError::User { error } => Err(error),
        },
    }
}
