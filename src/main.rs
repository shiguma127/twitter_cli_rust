#[macro_use]
extern crate clap;

use clap::{app_from_crate, Arg};
use egg_mode;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Config {
    api_key: String,
    api_secret_key: String,
    access_token: String,
    access_token_secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    // -m "path" -t "tweet text"
    let app = app_from_crate!()
        .arg(
            Arg::with_name("Text")
                .short("t")
                .long("Text")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Media")
                .short("m")
                .long("Media")
                .takes_value(true),
        );
    let matches = app.get_matches();
    let mut tweet_text: String = "tweet from rust ".into();
    if let Some(o) = matches.value_of("Text") {
        tweet_text = o.to_string();
    }
    let file = std::fs::read_to_string(get_config_path()?)?;
    let config: Config = toml::from_str(&file)?;
    let api_key = config.api_key;
    let api_secret_key = config.api_secret_key;
    let access_token = config.access_token;
    let access_token_secret = config.access_token_secret;
    let api_token = egg_mode::KeyPair::new(api_key, api_secret_key);
    let acc_token = egg_mode::KeyPair::new(access_token, access_token_secret);
    let token = egg_mode::Token::Access {
        consumer: api_token,
        access: acc_token,
    };
    let tweet = egg_mode::tweet::DraftTweet::new(tweet_text.clone());
    match tweet.send(&token).await {
        Ok(_) => println!("{}", "hi"),
        Err(err) => println!("{}", err),
    }
    //tweet.send(&token).await?;
    Ok(())
}
fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    match env::current_exe() {
        Ok(mut path) => {
            path.pop();
            path.push("Config.toml");
            Ok(path)
        }
        Err(_) => Err(format!("error occurred while getting current_exe"))?,
    }
}
