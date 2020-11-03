#[macro_use]
extern crate clap;
use clap::{app_from_crate, Arg};
use egg_mode::media::{get_status, media_types, upload_media, ProgressInfo};
use egg_mode::{self, error::Error};
use serde::Deserialize;
use std::env;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::delay_for;

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

    let mut tweet_text: String = " ".into();
    if let Some(o) = matches.value_of("Text") {
        tweet_text = o.to_string();
    }
    let mut tweet = egg_mode::tweet::DraftTweet::new(tweet_text.clone());
    if let Some(path) = matches.value_of("Media") {
        println!("{}", path);
        let media_path = PathBuf::from(path);
        println!("{:?}", media_path.extension());
        let typ = match media_path
            .extension()
            .and_then(|os| os.to_str())
            .unwrap_or("")
        {
            "jpg" | "jpeg" => media_types::image_jpg(),
            "gif" => media_types::image_gif(),
            "png" => media_types::image_png(),
            "webp" => media_types::image_webp(),
            "mp4" => media_types::video_mp4(),
            _ => {
                eprintln!("Format not recognized, must be one of [jpg, jpeg, gif, png, webp, mp4]");
                std::process::exit(1);
            }
        };
        let bytes = tokio::fs::read(path).await?;
        let handle = upload_media(&bytes, &typ, &token).await?;
        tweet.add_media(handle.id.clone());
        println!("Media uploaded");
        println!("Waiting for media to finish processing..");
        println!("{:?}", &handle.id);
        stdout().flush()?;
        for ct in 0..60 {
            let prog = get_status(handle.id.clone(), &token).await;
            if let Err(Error::TwitterError(_, _)) = prog {
                break;
            }

            match prog?.progress {
                None | Some(ProgressInfo::Success) => {
                    println!("\nMedia sucessfully processed");
                    break;
                }
                Some(ProgressInfo::Pending(_)) | Some(ProgressInfo::InProgress(_)) => {
                    print!(".");
                    delay_for(Duration::from_secs(1)).await;
                }
                Some(ProgressInfo::Failed(err)) => Err(err)?,
            }
            if ct == 60 {
                Err("Error: timeout")?
            }
        }
    }
    match tweet.send(&token).await {
        Ok(_) => {} //ignore
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
