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
} // pub fn parse(input: &str) -> IResult<&str, Vec<Line>>

fn is_not_line_ending(c: char) -> bool {
    c != '\n' && c != '\r'
} // fn is_not_line_ending(c: char) -> bool

fn is_not_line_ending_or_comment(c: char) -> bool {
    is_not_line_ending(c) && c != '#'
} // fn is_not-line_ending_or_comment(c: char) -> bool

fn is_carriage_return(c: char) -> bool {
    c == '\r'
} // fn is_carriage_return(c: char) -> bool

fn consume_newline(input: &str) -> IResult<&str, Option<&str>> {
    let (input, _) = take_while(is_carriage_return)(input)?;
    let (input, output) = opt(tag("\n"))(input)?;
    Ok((input, output))
} // fn consume_newline(input: &str) -> IResult<&str, Option<&str>>

fn product(input: &str) -> IResult<&str, &str> {
    let (input, _) = alt((preceded(space0, tag(":")), space1))(input)?;
    let (input, line) = take_while(is_not_line_ending_or_comment)(input)?;
    let (input, _) = opt(preceded(tag("#"), take_while(is_not_line_ending)))(input)?;
    let (input, _) = consume_newline(input)?;
    let line = line.trim();
    Ok((input, line))
} // fn product(input: &str) -> IResult<&str, &str>

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
} // fn parse_user_agent(input: &str) -> IResult<&str, Line>

fn parse_allow(input: &str) -> IResult<&str, Line> {
    let allow = (
        tag_no_case("allow"),
        tag_no_case("alow"),
        tag_no_case("alaw"),
        tag_no_case("allows"),
        tag_no_case("alows"),
        tag_no_case("alaws"),
    );

    let (input, _) = preceded(space0, alt(allow))(input)?;

    let (input, allow) = product(input)?;
    Ok((input, Line::Allow(allow)))
} // fn parse_allow(input: &str) -> IResult<&str, Line>

fn parse_disallow(input: &str) -> IResult<&str, Line> {
    let disallow = (
        tag_no_case("disallow"),
        tag_no_case("dissallow"),
        tag_no_case("dissalow"),
        tag_no_case("disalow"),
        tag_no_case("diasllow"),
        tag_no_case("disallaw"),
        tag_no_case("disallows"),
        tag_no_case("dissallows"),
        tag_no_case("dissalows"),
        tag_no_case("disalows"),
        tag_no_case("diasllows"),
        tag_no_case("disallaws"),
    );

    let (input, _) = preceded(space0, alt(disallow))(input)?;

    let (input, disallow) = product(input)?;
    Ok((input, Line::Disallow(disallow)))
} // fn parse_disallow(input: &str) -> IResult<&str, Line>

fn parse_sitemap(input: &str) -> IResult<&str, Line> {
    let sitemap = (
        tag_no_case("sitemap"),
        tag_no_case("site map"),
        tag_no_case("site-map"),
        tag_no_case("site maps"),
        tag_no_case("site-maps"),
        tag_no_case("site-maps"),
    );

    let (input, _) = preceded(space0, alt(sitemap))(input)?;

    let (input, sitemap) = product(input)?;
    Ok((input, Line::Sitemap(sitemap)))
} // fn parse_sitemap(input: &str) -> IResult<&str, Line>

fn parse_crawl_delay(input: &str) -> IResult<&str, Line> {
    let crawl_delay = (
        tag_no_case("crawl-delay"),
        tag_no_case("crawl delay"),
        tag_no_case("crawldelay"),
        tag_no_case("crawl delays"),
        tag_no_case("crawl-delays"),
        tag_no_case("crawl-delays"),
        tag_no_case("crawldelays"),
    );

    let (input, _) = preceded(space0, alt(crawl_delay))(input)?;

    let (input, crawl_delay) = product(input)?;

    let crawl_delay = crawl_delay.parse().ok().filter(|&x| x >= 0.0);

    Ok((input, Line::CrawlDelay(crawl_delay)))
} // fn parse_craw_delay

fn parse_raw(input: &str) -> IResult<&str, Line> {
    let (input, _raw) = take_while(is_not_line_ending)(input)?;
    let (input, _) = consume_newline(input)?;
    Ok((input, Line::Raw(())))
} // fn parse_raw
