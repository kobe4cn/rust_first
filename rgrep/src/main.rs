use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use dashmap::DashMap;

use regex::Regex;
use rgrep::GrepError;
use std::ops::Deref;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Rgrep {
    #[command(flatten)]
    inner: Inner,
}

#[derive(Debug, Parser)]
struct Inner {
    pattern: String,
    path: Vec<String>,
}

impl Deref for Rgrep {
    type Target = Inner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Rgrep {
    pub async fn match_with_default_strategy(&self) -> Result<(), GrepError> {
        // println!("pattern: {}, path: {:?}", rgrep.pattern, rgrep.path);
        let re = Regex::new(&self.pattern)?;
        let dashmap = DashMap::new();
        for fs in &self.path {
            let mut file = File::open(fs).await?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;
            let text = String::from_utf8(buf)?;
            let file_lines = format_line(text, &re);
            dashmap.insert(fs.clone(), file_lines);
        }
        self.print_result(&dashmap);
        Ok(())
    }

    pub fn print_result(&self, dashmap: &DashMap<String, Vec<String>>) {
        for entry in dashmap.iter() {
            println!("{}:", entry.key());
            let value = entry.value();
            for line in value.iter() {
                println!("    {}", line);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let rgrep = Rgrep::parse();
    rgrep.match_with_default_strategy().await?;
    Ok(())
}

pub fn format_line(text: String, re: &Regex) -> Vec<String> {
    let mut file_lines = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let mut line_format = String::new();
        if re.is_match(line) {
            let highlighted = re.replace_all(line, |caps: &regex::Captures| {
                if let Some(cap) = caps.get(0) {
                    let range = format!("{}:{}", i + 1, cap.start());
                    line_format = range.clone();
                }
                format!("{}", &caps[0].red())
            });
            file_lines.push(format!("{} {}", line_format, highlighted));
        }
    }
    file_lines
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     let rgrep = Rgrep::parse();
//     // println!("pattern: {}, path: {:?}", rgrep.pattern, rgrep.path);
//     let re = Regex::new(&rgrep.pattern)?;
//     let dashmap = DashMap::new();
//     for fs in &rgrep.path {
//         let mut file = File::open(fs).await?;
//         let mut buf = Vec::new();
//         file.read_to_end(&mut buf).await?;
//         let text = String::from_utf8(buf)?;
//         let mut file_lines = Vec::new();
//         for (i, line) in text.lines().enumerate() {
//             let mut line_format = String::new();
//             if re.is_match(line) {
//                 let highlighted = re.replace_all(line, |caps: &regex::Captures| {
//                     if let Some(cap) = caps.get(0) {
//                         let range = format!("{}:{}", i + 1, cap.start());
//                         line_format = range.clone();
//                     }
//                     format!("{}", &caps[0].red())
//                 });
//                 file_lines.push(format!("{} {}", line_format, highlighted));
//             }
//         }
//         dashmap.insert(fs.clone(), file_lines);
//     }
//     for entry in dashmap.iter() {
//         println!("{}:", entry.key());
//         let value = entry.value();
//         for line in value.iter() {
//             println!("    {}", line);
//         }
//     }
//     Ok(())
// }
