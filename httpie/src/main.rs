use std::str::FromStr;

use anyhow::Result;
use clap::{Parser, Subcommand};
use reqwest::Method;
#[derive(Parser,Debug)]
#[command(version = "1.0", author = "kevin.yang.xgz@gmail.com")]
struct Opts {
    #[command(subcommand)]
    subcmd: Commands,
}

#[derive(Subcommand,Debug)]
enum Commands {
    Get(Get),
    Post(Post),
}

#[derive(Parser,Debug)]
struct Get {
    #[arg(short, long)]
    url: String,
}

#[derive(Parser,Debug)]
struct Post {
    #[arg(short, long)]
    url: String,
    #[arg(short, long)]
    data: Vec<String>,
}


#[tokio::main]
async fn main()->Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        Commands::Get(get) => {
            println!("{}", get.url);
            // request(&get.url, "GET", &[]).await?;
        }
        Commands::Post(post) => {
            println!("{}", post.url);
            println!("{:?}", post.data);
            // request(&post.url, "POST", &post.data).await?;
        }
    }
    Ok(())
}
#[allow(dead_code)]
async fn request(url: &str, method: &str, data: &[String])->Result<()> {
    let client = reqwest::Client::new();
    let mut req = client.request(Method::from_str(method)?, url);
    req = req.header("Content-Type", "application/json");
    if !data.is_empty() {
        req = req.body(data.join("&"));
    }
    let res = req.send().await?;
    println!("{}", res.status());
    for (key, value) in res.headers() {
        println!("{}: {}", key, value.to_str()?);
    }
    if res.content_length().is_some() {
        let body = res.text().await?;
        println!("{}", jsonxf::pretty_print(&body).map_err(|e| anyhow::anyhow!(e))?);
    }
    Ok(())
}

