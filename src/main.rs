use std::{env, fs, io};
use std::error::Error;
use std::ops::Add;
use std::str::FromStr;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct Categories {
    value: Vec<String>,
}

impl FromStr for Categories {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: Vec<String> = s
            .split(",")
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|x: &str| x.to_string())
            .collect();

        Ok(Categories { value })
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// URL to the mastodon instance without uri scheme or trailing slash
    #[clap(short, long)]
    instance: String,

    /// Categories of emojis to download
    #[clap(short, long)]
    categories: Categories,
}

#[derive(Serialize, Deserialize, Debug)]
struct Emoji {
    shortcode: String,
    static_url: String,
    category: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();
    let emojis = reqwest::get("https://".to_string() + &*args.instance + "/api/v1/custom_emojis")
        .await?
        .json::<Vec<Emoji>>()
        .await?;

    let categories = args.categories;
    let current_dir = env::current_dir()?;
    let mut emojis_downloaded = 0;

    for emoji in emojis {
        if !categories.value.contains(&emoji.category) {
            continue;
        }

        let url = emoji.static_url;
        let pic_bytes = reqwest::get(&url).await?.bytes().await?;

        let name = emoji.shortcode;
        let file_type = url.split(".").last().ok_or(".png")?;

        let path = current_dir.as_path().join(emoji.category);
        fs::create_dir_all(&path);
        let file_path = &path.join(String::from(&*name).add(".").add(file_type));
        fs::write(&file_path, pic_bytes)?;

        emojis_downloaded = emojis_downloaded + 1;
        println!("Downloaded emoji {}", name);
    }

    println!("Downloaded a total of {} emojis", emojis_downloaded);
    Ok(())
}
