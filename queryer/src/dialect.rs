use sqlparser::dialect::Dialect;

#[derive(Debug, Default)]
pub struct TryDialect;

impl Dialect for TryDialect {
    fn is_identifier_start(&self, ident: char) -> bool {
        ('a'..='z').contains(&ident) || ('A'..='Z').contains(&ident) || ident == '_'
    }

    fn is_identifier_part(&self, ident: char) -> bool {
        ('a'..='z').contains(&ident)
            || ('A'..='Z').contains(&ident)
            || ('0'..='9').contains(&ident)
            || ['$', ':', '/', '&', '=', '-', '_', '.', '?'].contains(&ident)
    }
}

#[allow(dead_code)]
pub fn example_sql() -> String {
    let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
    let sql = format!(
        "select location,total_cases,new_cases,total_deaths,new_deaths from {url} where new_deaths>100 order by new_deaths desc"
    );
    sql
}

#[cfg(test)]
mod tests {
    use sqlparser::parser::Parser;

    use super::*;

    #[test]
    fn test_example_sql() {
        assert!(Parser::parse_sql(&TryDialect::default(), &example_sql()).is_ok());
    }
}
