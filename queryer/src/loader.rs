use std::io::Cursor;

use crate::DataSet;
use anyhow::Result;
use polars::{io::SerReader, prelude::CsvReader};

pub trait Load {
    type Error;
    fn load(&self) -> Result<DataSet, Self::Error>;
}

#[derive(Debug, Clone)]
pub enum Loader {
    Csv(CsvLoader),
}

#[derive(Debug, Clone)]
pub struct CsvLoader(pub(crate) String);

impl Loader {
    pub fn load(&self) -> Result<DataSet> {
        match self {
            Loader::Csv(loader) => loader.load(),
        }
    }
}

pub fn detect_content(data: String) -> Loader {
    Loader::Csv(CsvLoader(data))
}

impl Load for CsvLoader {
    type Error = anyhow::Error;
    fn load(&self) -> Result<DataSet, Self::Error> {
        let df = CsvReader::new(Cursor::new(&self.0)).finish()?;
        Ok(DataSet(df))
    }
}

#[cfg(test)]
mod tests {
    use polars::prelude::{IntoLazy, col};

    use super::*;

    #[tokio::test]
    async fn test_detect_content() -> Result<()> {
        let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
        let data = reqwest::get(url).await?.text().await?;
        let data_set = detect_content(data.to_string()).load()?;
        println!("{:#?}", data_set.0.lazy().collect()?);
        Ok(())
    }
}
