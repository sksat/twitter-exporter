use std::fs;
use std::io::Read;

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub port: u16,
    pub cron: String,
    pub token: Token,
}

#[derive(Deserialize)]
pub struct Token {
    consumer_key: String,
    consumer_secret: String,
    access_token: String,
    access_secret: String,
}

pub fn load(file: &str) -> Result<Config, std::io::Error> {
    let mut f = fs::File::open(file)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    let cfg: Config = toml::from_str(&content)?;
    Ok(cfg)
}

impl From<Token> for egg_mode::auth::Token {
    fn from(t: Token) -> Self {
        let consumer = egg_mode::KeyPair::new(t.consumer_key, t.consumer_secret);
        let access = egg_mode::KeyPair::new(t.access_token, t.access_secret);
        egg_mode::Token::Access { consumer, access }
    }
}
