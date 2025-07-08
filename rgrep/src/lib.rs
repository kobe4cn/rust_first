mod error;

use std::ops::Range;
use std::path::Path;

use colored::Colorize;
pub use error::GrepError;

use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Stdout};

#[derive(Debug, Parser)]
pub struct GrepConfig {
    pub pattern: String,
    pub path: Vec<String>,
}

pub type StrategyFn<W, R> = fn(&Path, BufReader<R>, &Regex, &mut W) -> Result<(), GrepError>;

impl GrepConfig {
    pub fn match_with_default_strategy(&self) -> Result<(), GrepError> {
        self.match_with_strategy(default_strategy)
    }
    pub fn match_with_strategy(&self, strategy: StrategyFn<Stdout, File>) -> Result<(), GrepError> {
        let re = Regex::new(&self.pattern)?;
        let files = &self.path;
        files.into_iter().for_each(|filename| {
            if let Ok(file) = File::open(filename) {
                let reader = BufReader::new(file);
                let mut stdout = std::io::stdout();
                if let Err(e) = strategy(Path::new(filename), reader, &re, &mut stdout) {
                    println!("Internal error: {}", e);
                }
            }
        });
        Ok(())
    }
}

pub fn default_strategy<W: std::io::Write, R: Read>(
    path: &Path,
    reader: BufReader<R>,
    re: &Regex,
    stdout: &mut W,
) -> Result<(), GrepError> {
    let matches: Vec<String> = reader
        .lines()
        .enumerate()
        .map(|(line_number, line)| {
            line.ok()
                .map(|line| {
                    re.find(&line)
                        .map(|mat| format_line(&line, line_number + 1, mat.range()))
                })
                .flatten()
        })
        .filter_map(|v| v.ok_or(()).ok())
        .collect();

    let result = matches.join("\n");
    if !result.is_empty() {
        stdout.write(path.display().to_string().as_bytes())?;
        stdout.write(b"\n")?;
        stdout.write(result.as_bytes())?;
        stdout.write(b"\n")?;
    }

    Ok(())
}

pub fn format_line(line: &str, lineno: usize, range: Range<usize>) -> String {
    let Range { start, end } = range;
    let prefix = &line[..start];
    format!(
        "{0: >6}:{1: <3} {2}{3}{4}",
        lineno.to_string().blue(),
        (prefix.chars().count() + 1).to_string().cyan(),
        prefix,
        &line[start..end].red(),
        &line[end..]
    )
}
