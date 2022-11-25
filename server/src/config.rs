use std::net::IpAddr;

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub database_url: String,
    pub log_level: String,
    pub host: IpAddr,
    pub port: u16,
    pub user_agent: String,
    pub assets_path: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            database_url: "sqlite://db.sqlite".to_string(),
            log_level: "info".to_string(),
            host: "::".parse().expect("invalid default host"),
            port: 8080,
            user_agent: "podreplay.com".to_string(),
            assets_path: "ui".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> std::result::Result<Config, figment::Error> {
        Figment::from(Serialized::defaults(Config::default()))
            .merge(Toml::file("podreplay.toml"))
            .merge(Env::prefixed("PODREPLAY_"))
            .extract()
    }
}
