#[macro_use]
extern crate clap;

use clap::{app_from_crate, Arg};
use egg_mode;
use std::env;

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
    let api_key = env::var("API_key").expect("not exist");
    let api_secret_key = env::var("API_Secret_Key").expect("not exist");
    let access_token = env::var("Access_Token").expect("not exist");
    let access_token_secret = env::var("Access_Token_Secret").expect("not exist");
    let api_token = egg_mode::KeyPair::new(api_key, api_secret_key);
    let acc_token = egg_mode::KeyPair::new(access_token, access_token_secret);
    let token = egg_mode::Token::Access {
        consumer: api_token,
        access: acc_token,
    };
    let tweet = egg_mode::tweet::DraftTweet::new(tweet_text.clone());
    tweet.send(&token).await?;
    Ok(())
}
