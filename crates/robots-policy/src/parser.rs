/**
 * @file parser.rs
 * @author Krisna Pranav
 * @brief robots-policy[parser]
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while},
    character::complete::{space0, space1},
    combinator::{eof, opt},
    multi::many_till,
    sequence::preceded,
    IResult,
};

#[derive(Debug)]
pub enum Line<'a> {
    UserAgent(Vec<&'a str>),
    Allow(&'a str),
    Disallow(&'a str),
    Sitemap(&'a str),
    CrawlDelay(Option<f32>),
    Raw(()),
} // pub enum Line<'a>

pub fn parse(input: &str) -> IResult<&str, Vec<Line>> {
    let (input, (lines, _)) = many_till(
        alt((
            parse_user_agent,
            parse_allow,
            parse_disallow,
            parse_sitemap,
            parse_crawl_delay,
            parse_raw,
        )),
        eof,
    )(input)?;

    Ok((input, lines))
}

fn is_not_line_ending(c: char) -> bool {
    c != '\n' && c != '\r'
}

fn is_not_line_ending_or_comment(c: char) -> bool {
    is_not_line_ending(c) && c != '#'
}

fn is_carriage_return(c: char) -> bool {
    c == '\r'
}

fn consume_newline(input: &str) -> IResult<&str, Option<&str>> {
    let (input, _) = take_while(is_carriage_return)(input)?;
    let (input, output) = opt(tag("\n"))(input)?;
    Ok((input, output))
}

fn product(input: &str) -> IResult<&str, &str> {
    let (input, _) = alt((preceded(space0, tag(":")), space1))(input)?;
    let (input, line) = take_while(is_not_line_ending_or_comment)(input)?;
    let (input, _) = opt(preceded(tag("#"), take_while(is_not_line_ending)))(input)?;
    let (input, _) = consume_newline(input)?;
    let line = line.trim();
    Ok((input, line))
}

fn parse_user_agent(input: &str) -> IResult<&str, Line> {
    let useragent = (
        tag_no_case("user-agent"),
        tag_no_case("user agent"),
        tag_no_case("useragent"),
        tag_no_case("user-agents"),
        tag_no_case("user agents"),
        tag_no_case("useragents"),
    );
    let (input, _) = preceded(space0, alt(useragent))(input)?;
    let (input, user_agents) = product(input)?;

    let user_agents = user_agents
        .split_whitespace()
        .flat_map(|x| x.split(','))
        .collect();

    Ok((input, Line::UserAgent(user_agents)))
}
