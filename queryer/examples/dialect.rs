use anyhow::Result;
use sqlparser::{dialect::GenericDialect, parser::Parser};
#[tokio::main]
async fn main() -> Result<()> {
    let sql = "select a a1, b, 123, myfunc(b), * from data_source where a > b \
        and b<100 and c between 10 and 20 \
    order by a desc, b limit 50 offset 10";

    let ast = Parser::parse_sql(&GenericDialect::default(), sql)?;
    println!("ast: {:#?} \n", ast);
    let sql_str = ast.iter().map(|stmt| stmt.to_string()).collect::<Vec<_>>();
    println!("sql: {}", sql_str[0]);
    Ok(())
}
