use std::ops::{Deref, DerefMut};

use polars::{
    frame::DataFrame,
    io::SerWriter,
    prelude::{CsvWriter, IntoLazy, PlSmallStr, SortMultipleOptions},
};

mod convert;
mod dialect;
mod fetcher;
mod loader;
use anyhow::Result;
use sqlparser::parser::Parser;
use tracing::info;

use crate::{convert::Sql, dialect::TryDialect, fetcher::retrieve_data, loader::detect_content};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
#[derive(Debug, Clone)]
pub struct DataSet(DataFrame);

impl Deref for DataSet {
    type Target = DataFrame;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DataSet {
    pub fn to_csv(&mut self) -> Result<String> {
        let mut buffer = Vec::new();
        let mut writer = CsvWriter::new(&mut buffer);
        writer.finish(&mut self.0)?;
        Ok(String::from_utf8(buffer)?)
    }
}

pub async fn query<T: AsRef<str>>(sql: T) -> Result<DataSet> {
    let ast = Parser::parse_sql(&TryDialect::default(), sql.as_ref())?;
    if ast.len() != 1 {
        return Err(anyhow::anyhow!(
            "Only support single SQL statement: {}",
            sql.as_ref()
        ));
    }
    let sql = &ast[0];
    let Sql {
        selection,
        source,
        conditions,
        order_by,
        limit,
        offset,
    } = sql.try_into()?;
    info!("retrieving data from {}", source);

    let ds = detect_content(retrieve_data(source).await?).load()?;
    let mut ds_filtered = match conditions {
        Some(expr) => ds.0.lazy().filter(expr),
        None => ds.0.lazy(),
    };
    ds_filtered = order_by.into_iter().fold(ds_filtered, |acc, (expr, desc)| {
        acc.sort(
            vec![PlSmallStr::from_str(expr.as_str())],
            SortMultipleOptions::default().with_order_descending(desc),
        )
    });
    if offset.is_some() || limit.is_some() {
        ds_filtered = ds_filtered.slice(offset.unwrap_or(0), limit.unwrap_or(usize::MAX) as u32);
    }
    let data_set = ds_filtered.select(selection).collect()?;

    Ok(DataSet(data_set))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
