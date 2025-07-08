use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use dashmap::DashMap;

use futures::{StreamExt, stream::FuturesUnordered};
use regex::Regex;
use rgrep::{GrepConfig, GrepError};
use std::{io::Write, ops::Deref};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

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
#[allow(unused)]
impl Rgrep {
    pub async fn match_with_default_strategy(&self) -> Result<(), GrepError> {
        // println!("pattern: {}, path: {:?}", rgrep.pattern, rgrep.path);
        let re = Regex::new(&self.pattern)?;
        //实现多个文件同时异步搜索
        let mut tasks = FuturesUnordered::new();
        let dashmap = DashMap::new();

        for fs in &self.path {
            let fs_clone = fs.clone();
            let re_clone = re.clone();
            
            tasks.push(tokio::spawn(async move {
                search_in_file(fs_clone, re_clone).await
            }));
        }

        while let Some(Ok(result)) = tasks.next().await {
            let (path, lines) = result?;
            dashmap.insert(path, lines);
        }
        let stdout = std::io::stdout();
        self.print_result(&dashmap, stdout);
        Ok(())
    }

    pub fn print_result<W: Write>(
        &self,
        dashmap: &DashMap<String, Vec<String>>,
        mut stdout: W,
    ) -> Result<(), GrepError> {
        for entry in dashmap.iter() {
            writeln!(stdout, "{}:", entry.key())?;
            for line in entry.value().iter() {
                writeln!(stdout, " {}", line)?;
            }
        }
        Ok(())
    }

    // pub fn print_result(&self, dashmap: &DashMap<String, Vec<String>>) {
    //     for entry in dashmap.iter() {
    //         println!("{}:", entry.key());
    //         let value = entry.value();
    //         for line in value.iter() {
    //             println!("    {}", line);
    //         }
    //     }
    // }
}

async fn search_in_file(path: String, re: Regex) -> Result<(String, Vec<String>), GrepError> {
    let file = File::open(&path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut result = Vec::new();
    let mut lineno = 0;
    while let Some(line) = lines.next_line().await? {
        lineno += 1;
        let mut line_format = String::new();
        if re.is_match(&line) {
            let highlighted = re.replace_all(&line, |caps: &regex::Captures| {
                if let Some(cap) = caps.get(0) {
                    line_format = format!("{}:{}", lineno, cap.start());
                }
                format!("{}", &caps[0].red())
            });
            result.push(format!("{}{}", line_format, highlighted));
        }
    }
    Ok((path, result))
}

// pub fn format_line(text: String, re: &Regex) -> Vec<String> {
//     let mut file_lines = Vec::new();
//     for (i, line) in text.lines().enumerate() {
//         let mut line_format = String::new();
//         if re.is_match(line) {
//             let highlighted = re.replace_all(line, |caps: &regex::Captures| {
//                 if let Some(cap) = caps.get(0) {
//                     let range = format!("{}:{}", i + 1, cap.start());
//                     line_format = range.clone();
//                 }
//                 format!("{}", &caps[0].red())
//             });
//             file_lines.push(format!("{} {}", line_format, highlighted));
//         }
//     }
//     file_lines
// }

// #[tokio::main]
// async fn main() -> Result<()> {
//     let rgrep = Rgrep::parse();
//     rgrep.match_with_default_strategy().await?;
//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<()> {
    let rgrep = GrepConfig::parse();
    rgrep.match_with_default_strategy()?;
    Ok(())
}
//最简单的写法，但是效率很低，因为每次都要读取文件，然后匹配，然后输出
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
