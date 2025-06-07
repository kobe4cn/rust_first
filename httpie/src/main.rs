use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use reqwest::{Method, Url};
use serde::Serialize;
use serde_json;

use std::{collections::HashMap, str::FromStr};
#[derive(Parser, Debug)]
#[command(version = "1.0", author = "kevin.yang.xgz@gmail.com")]
struct Opts {
    #[command(subcommand)]
    subcmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Get(Get),
    Post(Post),
}

#[derive(Parser, Debug)]
struct Get {
    #[arg(short, long, value_parser = parse_url)]
    url: String,
}

#[derive(Parser, Debug)]
struct Post {
    #[arg(short, long, value_parser = parse_url)]
    url: String,
    #[arg(short, long, value_parser = parse_data)]
    data: Vec<KeyPair>,
}

#[derive(Debug, Clone, Serialize)]
struct KeyPair {
    key: String,
    value: String,
}

impl FromStr for KeyPair {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, value) = s
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("failed to split key-value"))?;
        Ok(KeyPair {
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}

fn parse_data(data: &str) -> Result<KeyPair> {
    let key_pair = data.parse::<KeyPair>()?;
    Ok(key_pair)
}

fn parse_url(url: &str) -> Result<String> {
    let new_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("http://{}", url)
    };
    let _url: Url = new_url.parse()?;
    Ok(new_url.into())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        Commands::Get(get) => {
            request(&get.url, "GET", &[]).await?;
        }
        Commands::Post(post) => {
            request(&post.url, "POST", &post.data).await?;
        }
    }
    Ok(())
}
#[allow(dead_code)]
async fn request(url: &str, method: &str, data: &[KeyPair]) -> Result<()> {
    let client = reqwest::Client::new();
    let mut req = client.request(Method::from_str(method)?, url);

    req = req.header("Content-Type", "application/json");
    if !data.is_empty() {
        let mut map = HashMap::new();
        for pair in data {
            map.insert(&pair.key, &pair.value);
        }
        let json_data = serde_json::to_string(&map).map_err(|e| anyhow::anyhow!(e))?;
        req = req.body(json_data);
    }
    let res = req.send().await?;
    let version = format!("{:?}", res.version()).blue();
    let status = res.status().to_string();
    let (code, str) = status
        .split_once(" ")
        .ok_or_else(|| anyhow::anyhow!("failed to split status"))?;
    println!(
        "{} {} {}",
        version,
        code.to_string().blue(),
        str.to_string().red()
    );
    for (key, value) in res.headers() {
        println!("{}: {}", key.to_string().green(), value.to_str()?.purple());
    }
    if res.content_length().is_some() {
        let body = res.text().await?;
        let formatted = jsonxf::pretty_print(&body).map_err(|e| anyhow::anyhow!(e))?;
        println!("{}", color_json(&formatted));
    }
    Ok(())
}

fn color_json(json_str: &str) -> String {
    json_str
        .lines()
        .map(|line| {
            if let Some((key, value)) = line.split_once(':') {
                format!("{}{}{}", key.red(), ":".to_string(), value.yellow())
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}
