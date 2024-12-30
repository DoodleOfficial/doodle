/**
 * @file engine.rs
 * @author Krisna Pranav
 * @brief engine
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use kuchiki::traits::*;
use std::sync::LazyLock;

static PARSED_CONFIG: LazyLock<Config> =
    LazyLock::new(|| toml::from_str(include_str!("conf.toml")).unwrap());

#[derive(serde::Deserialize, Clone)]
pub struct Config {
    engines: Vec<EngineConf>,
}

impl Config {
    pub fn new() -> Self {
        PARSED_CONFIG.clone()
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct EngineConf {
    name: String,
    search: String,
    xpath: String,
}

pub struct Engine {
    conf: EngineConf,
}

impl Engine {
    pub fn new(conf: EngineConf) -> Self {
        Engine { conf }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        let conf = Config::new().engines.into_iter().find(|e| e.name == name)?;
        Some(Engine::new(conf))
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Url>, reqwest::Error> {
        let encoded_query = query.replace(' ', "+");
        let url = self.conf.search.replace("{query}", &encoded_query);
        let useragent = crate::useragent::UserAgent::random_weighted();

        let client = reqwest::Client::builder()
            .user_agent(useragent.as_str())
            .build()?;

        let body = client.get(&url).send().await?.text().await?;

        Ok(kuchiki::parse_html()
            .one(body)
            .select_xpath(&self.conf.xpath)
            .map(|url| {
                let attr = url.attributes.borrow();
                let url = attr.get("href").unwrap();
                Url(url.to_string())
            })
            .collect())
    }
}

#[derive(Debug)]
pub struct Url(String);

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
