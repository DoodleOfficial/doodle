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
use crate::parser::Line;
use itertools::Itertools;
use pattern::Pattern;
use std::time::Duration;
use url::Url;

#[derive(Debug, PartialEq, Eq)]
enum Directive {
    Allow,
    Disallow,
} // enum Directive

impl Ord for Directive {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Allow, Self::Disallow) => std::cmp::Ordering::Less,
            (Self::Disallow, Self::Allow) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    }
} // impl Ord for Directive

impl PartialOrd for Directive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
} // impl PartialOrd for Directive

#[derive(Debug, PartialEq, Eq)]
struct Rule {
    pattern: Pattern,
    directive: Directive,
} // struct Rule

impl Ord for Rule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pattern
            .cmp(&other.pattern)
            .then(self.directive.cmp(&other.directive))
    } // fn cmp(&self, other: &Self) -> std::cmp::Ordering
} // impl Ord for Rule

impl PartialOrd for Rule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    } // fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
} // impl PartialOrd for Rule

#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub char_limit: usize,
} // pub struct Params

impl Default for Params {
    fn default() -> Self {
        Self {
            char_limit: MAX_CHAR_LIMIT_DEFAULT,
        }
    } // fn default() -> Self
} // impl Default for Params

#[derive(Debug)]
pub struct Robots {
    rules: Vec<Rule>,
    crawl_delay: Option<f32>,
    sitemaps: Vec<String>,
} // pub struct Robots

impl Robots {
    fn is_valid_user_agent(useragent: &str) -> bool {
        useragent
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '-' || c == '_')
            && !useragent.is_empty()
    } // fn is_valid_user_agent(useragent: &str) -> bool

    pub fn parse_with_params(
        useragent: &str,
        robotstxt: &str,
        params: Params,
    ) -> Result<Self, anyhow::Error> {
        if !Self::is_valid_user_agent(useragent) {
            return Err(anyhow::anyhow!("Invalid user agent"));
        }

        let robotstxt = robotstxt
            .chars()
            .take(params.char_limit)
            .collect::<String>();

        let robotstxt = robotstxt.replace('\0', "\n");
        let (_, lines) = parser::parse(&robotstxt).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let mut useragent = useragent.to_lowercase();

        if !lines.iter().any(|line| {
            if let Line::UserAgent(agents) = line {
                agents.iter().any(|agent| {
                    agent
                        .chars()
                        .zip(useragent.chars())
                        .all(|(c1, c2)| c1.to_ascii_lowercase() == c2)
                })
            } else {
                false
            }
        }) {
            useragent = "*".to_string();
        }

        let mut rules = Vec::new();
        let mut crawl_delay = None;
        let mut sitemaps = Vec::new();
        let mut idx = 0;
        let mut useragent_lines = 0;

        while idx < lines.len() {
            let line = &lines[idx];
            idx += 1;

            if let Line::UserAgent(agents) = &line {
                useragent_lines += 1;
                if agents.iter().any(|agent| {
                    agent
                        .chars()
                        .zip(useragent.chars())
                        .all(|(c1, c2)| c1.to_ascii_lowercase() == c2)
                }) {
                    let mut has_captured_directive = false;
                    while idx < lines.len() {
                        let line = &lines[idx];
                        idx += 1;

                        match line {
                            Line::Allow(path) => {
                                has_captured_directive = true;

                                if !path.is_empty() {
                                    rules.push(Rule {
                                        pattern: Pattern::new(path),
                                        directive: Directive::Allow,
                                    });
                                }
                            }
                            Line::Disallow(path) => {
                                has_captured_directive = true;

                                if !path.is_empty() {
                                    rules.push(Rule {
                                        pattern: Pattern::new(path),
                                        directive: Directive::Disallow,
                                    });
                                }
                            }
                            Line::UserAgent(_) if has_captured_directive => {
                                idx -= 1;
                                break;
                            }
                            Line::Sitemap(sitemap) => {
                                sitemaps.push(sitemap.to_string());
                            }
                            Line::CrawlDelay(Some(delay)) => {
                                has_captured_directive = true;
                                crawl_delay = Some(*delay);
                            }
                            _ => {}
                        }
                    }
                }
            } else if useragent_lines == 0 {
                match line {
                    Line::Allow(path) => {
                        rules.push(Rule {
                            pattern: Pattern::new(path),
                            directive: Directive::Allow,
                        });
                    }
                    Line::Disallow(path) => {
                        rules.push(Rule {
                            pattern: Pattern::new(path),
                            directive: Directive::Disallow,
                        });
                    }
                    Line::CrawlDelay(Some(delay)) => {
                        crawl_delay = Some(*delay);
                    }
                    _ => {}
                }
            }

            if let Line::Sitemap(url) = line {
                sitemaps.push(url.to_string());
            }
        }

        Ok(Self {
            rules,
            crawl_delay,
            sitemaps,
        })
    } // pub fn parse_with_params

    pub fn parse(useragent: &str, robotstxt: &str) -> Result<Self, anyhow::Error> {
        Self::parse_with_params(useragent, robotstxt, Params::default())
    } // pub fn parse(useragent: &str, robotstxt: &str) -> Result<Self, anyhow::Error>

    pub fn is_allowed(&self, url: &Url) -> bool {
        let path = &Self::prepare_path(url);
        self.is_path_allowed(path)
    } // pub fn is_allowed(&self, url: &Url) -> bool

    fn prepare_path(url: &Url) -> String {
        let path = url.path();

        let path = path
            .chars()
            .coalesce(|a, b| {
                if a == '/' && b == '/' {
                    Ok(a)
                } else {
                    Err((a, b))
                }
            })
            .collect::<String>();

        if let Some(query) = url.query() {
            format!("{}?{}", path, query)
        } else {
            path
        }
    } // fn prepare_path(url: &Url) -> String

    fn is_precise_path_allowed(&self, path: &str) -> bool {
        let mut path = path.to_string();

        if path.is_empty() {
            path = "/".to_string();
        }

        if path == "/robots.txt" {
            return true;
        }

        let mut matches: Vec<_> = self
            .rules
            .iter()
            .filter(|rule| rule.pattern.matches(&path))
            .collect();

        matches.sort();

        matches
            .first()
            .map(|rule| rule.directive == Directive::Allow)
            .unwrap_or(true)
    } // fn is_precise_path_allowed(&self, path: &str) -> bool

    pub fn is_path_allowed(&self, path: &str) -> bool {
        let res = self.is_precise_path_allowed(path);

        if !res && path.ends_with('/') {
            self.is_precise_path_allowed(format!("{}index.html", path).as_str())
        } else {
            res
        }
    } // pub fn is_path_allowed(&self, path: &str) -> bool

    pub fn crawl_delay(&self) -> Option<Duration> {
        self.crawl_delay.map(Duration::from_secs_f32)
    } // pub fn craw_delay(&self) -> Option<Duration>

    pub fn sitemaps(&self) -> &[String] {
        &self.sitemaps
    } // pub fn sitemaps(&self) -> &[String]
} // impl Robots
