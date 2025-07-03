use anyhow::Result;

use queryer::query;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
    let sql = format!(
        "select location,total_cases,new_cases,total_deaths,new_deaths from {url} where new_deaths>100 order by new_deaths desc"
    );
    // let sql = example_sql();
    let mut ds = query(&sql).await?;
    println!("{:?}", ds.head(Some(5)));

    println!("{}", ds.to_csv()?);

    Ok(())
}
