use std::sync::Arc;

use anyhow::Ok;
use polars::prelude::{
    AnyValue, DataType, Expr, LiteralValue, Operator, PlSmallStr, Scalar, col, is_not_null, is_null,
};
use polars_plan::plans::DynLiteralValue;
use sqlparser::ast::{
    BinaryOperator as SqlBinaryOperator, Expr as SqlExpr, Offset as SqlOffset, OrderByExpr,
    OrderByKind, Select, SelectItem, SetExpr, Statement, TableFactor, TableWithJoins,
    Value as SqlValue, ValueWithSpan,
};
#[derive(Debug)]
#[allow(dead_code)]
pub struct Sql<'a> {
    pub(crate) selection: Vec<Expr>,
    pub(crate) source: &'a str,
    pub(crate) conditions: Option<Expr>,
    pub(crate) order_by: Vec<(String, bool)>,
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<i64>,
}

pub struct Expression(pub(crate) Box<SqlExpr>);
pub struct Operation(pub(crate) SqlBinaryOperator);

pub struct Projection<'a>(pub(crate) &'a SelectItem);
pub struct Source<'a>(pub(crate) &'a [TableWithJoins]);
pub struct Order<'a>(pub(crate) &'a OrderByExpr);
pub struct Offset<'a>(pub(crate) &'a SqlOffset);
pub struct Limit<'a>(pub(crate) &'a SqlExpr);
pub struct Value(pub(crate) SqlValue);

impl<'a> TryFrom<&'a Statement> for Sql<'a> {
    type Error = anyhow::Error;

    fn try_from(sql: &'a Statement) -> Result<Self, Self::Error> {
        match sql {
            Statement::Query(q) => {
                let orders = &q.order_by;

                let (offset, limit) = match &q.limit_clause {
                    Some(l) => match l {
                        sqlparser::ast::LimitClause::LimitOffset {
                            limit,
                            offset,
                            limit_by: _,
                        } => (offset.as_ref(), limit.as_ref()),
                        _ => return Err(anyhow::anyhow!("Only support LimitOffset")),
                    },
                    None => (None, None),
                };

                let Select {
                    from: table_with_joins,
                    selection: where_clause,
                    projection,
                    ..
                } = match q.body.as_ref() {
                    SetExpr::Select(statement) => statement.as_ref(),
                    _ => return Err(anyhow::anyhow!("Only support Query")),
                };

                let source: &str = Source(table_with_joins).try_into()?;
                let conditions: Option<Expr> = where_clause
                    .as_ref()
                    .map(|expr| Expression(Box::new(expr.clone())).try_into())
                    .transpose()?;
                let mut selection = Vec::new();
                for item in projection {
                    let expr: Expr = Projection(item).try_into()?;
                    selection.push(expr);
                }
                let mut order_by_vec = Vec::new();
                if let Some(item) = orders {
                    match &item.kind {
                        OrderByKind::Expressions(exprs) => {
                            for expr in exprs {
                                let (name, asc) = Order(&expr).try_into()?;
                                order_by_vec.push((name, asc));
                            }
                        }
                        _expr => return Err(anyhow::anyhow!("Only support expressions")),
                    }
                }
                let offset = offset.map(|o| Offset(o).into());
                let limit = limit.map(|l| Limit(l).into());
                Ok(Sql {
                    selection,
                    source,
                    conditions,
                    order_by: order_by_vec,
                    limit,
                    offset,
                })
            }
            _ => return Err(anyhow::anyhow!("Not a select statement")),
        }
    }
}

impl<'a> TryFrom<Source<'a>> for &'a str {
    type Error = anyhow::Error;

    fn try_from(source: Source<'a>) -> Result<Self, Self::Error> {
        if source.0.len() != 1 {
            return Err(anyhow::anyhow!("Only support one source"));
        }
        let table = &source.0[0];
        if !table.joins.is_empty() {
            return Err(anyhow::anyhow!("Only support one source with no joins"));
        }

        match &table.relation {
            TableFactor::Table { name, .. } => {
                let name = name.0.first().unwrap();
                let table = name.as_ident().unwrap().value.as_str();
                Ok(table)
            }
            _ => Err(anyhow::anyhow!("Only support table")),
        }
    }
}
/// 把 SqlParser 的 Expr 转换成 DataFrame 的 Expr
impl TryFrom<Expression> for Expr {
    type Error = anyhow::Error;

    fn try_from(expr: Expression) -> Result<Self, Self::Error> {
        match *expr.0 {
            SqlExpr::BinaryOp { left, right, op } => {
                let left = Arc::new(Expression(left).try_into()?);
                let right = Arc::new(Expression(right).try_into()?);
                let op = Operation(op).try_into()?;
                Ok(Expr::BinaryExpr { left, op, right })
            }
            SqlExpr::Wildcard(_tok) => Ok(Self::Wildcard),
            SqlExpr::IsNull(expr) => Ok(is_null(Expression(expr).try_into()?)),
            SqlExpr::IsNotNull(expr) => Ok(is_not_null(Expression(expr).try_into()?)),
            SqlExpr::Identifier(id) => Ok(col(&id.to_string())),
            SqlExpr::Value(v) => Ok(Self::Literal(Value(v.value).try_into()?)),
            v => Err(anyhow::anyhow!("expr {:?} is not supported", v)),
        }
    }
}

impl TryFrom<Operation> for Operator {
    type Error = anyhow::Error;

    fn try_from(op: Operation) -> Result<Self, Self::Error> {
        match op.0 {
            SqlBinaryOperator::Plus => Ok(Self::Plus),
            SqlBinaryOperator::Minus => Ok(Self::Minus),
            SqlBinaryOperator::Multiply => Ok(Self::Multiply),
            SqlBinaryOperator::Divide => Ok(Self::Divide),
            SqlBinaryOperator::Modulo => Ok(Self::Modulus),
            SqlBinaryOperator::Gt => Ok(Self::Gt),
            SqlBinaryOperator::Lt => Ok(Self::Lt),
            SqlBinaryOperator::LtEq => Ok(Self::LtEq),
            SqlBinaryOperator::GtEq => Ok(Self::GtEq),
            SqlBinaryOperator::Eq => Ok(Self::Eq),
            SqlBinaryOperator::NotEq => Ok(Self::NotEq),
            SqlBinaryOperator::And => Ok(Self::And),
            SqlBinaryOperator::Or => Ok(Self::Or),

            v => Err(anyhow::anyhow!("Operator {} is not support", v)),
        }
    }
}

impl<'a> TryFrom<Projection<'a>> for Expr {
    type Error = anyhow::Error;

    fn try_from(projection: Projection<'a>) -> Result<Self, Self::Error> {
        match projection.0 {
            SelectItem::UnnamedExpr(SqlExpr::Identifier(id)) => Ok(col(&id.to_string())),
            SelectItem::ExprWithAlias {
                expr: SqlExpr::Identifier(id),
                alias,
            } => Ok(Expr::Alias(
                Arc::new(Expr::Column(PlSmallStr::from_str(&id.to_string()))),
                PlSmallStr::from_str(&alias.to_string()),
            )),
            SelectItem::QualifiedWildcard(_, _) => {
                Err(anyhow::anyhow!("QualifiedWildcard is not support"))
            }
            SelectItem::Wildcard(_) => Err(anyhow::anyhow!("Wildcard is not support")),
            item => Err(anyhow::anyhow!("Unsupported item: {:?}", item)),
        }
    }
}

impl<'a> TryFrom<Order<'a>> for (String, bool) {
    type Error = anyhow::Error;

    fn try_from(order: Order<'a>) -> Result<Self, Self::Error> {
        let name = match &order.0.expr {
            SqlExpr::Identifier(id) => id.to_string(),
            expr => return Err(anyhow::anyhow!("Only support identifier, got {:?}", expr)),
        };
        let asc = order.0.options.asc.unwrap_or(true);
        Ok((name, asc))
    }
}

/// 把 SqlParser 的 Limit expr 转换成 usize
impl<'a> From<Limit<'a>> for usize {
    fn from(l: Limit<'a>) -> Self {
        match l.0 {
            SqlExpr::Value(ValueWithSpan {
                value: SqlValue::Number(v, _b),
                span: _,
            }) => v.parse().unwrap_or(usize::MAX),
            _ => usize::MAX,
        }
    }
}

/// 把 SqlParser 的 offset expr 转换成 i64
impl<'a> From<Offset<'a>> for i64 {
    fn from(offset: Offset) -> Self {
        match offset.0 {
            SqlOffset {
                value:
                    SqlExpr::Value(ValueWithSpan {
                        value: SqlValue::Number(v, _b),
                        span: _,
                    }),
                ..
            } => v.parse().unwrap_or(0),
            _ => 0,
        }
    }
}

/// 把 SqlParser 的 value 转换成 DataFrame 支持的 LiteralValue
impl TryFrom<Value> for LiteralValue {
    type Error = anyhow::Error;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.0 {
            SqlValue::Number(v, _) => Ok(LiteralValue::Dyn(DynLiteralValue::Float(
                v.parse().unwrap(),
            ))),
            SqlValue::Boolean(v) => {
                let scaler = Scalar::new(DataType::Boolean, AnyValue::Boolean(v));
                Ok(LiteralValue::Scalar(scaler))
            }
            SqlValue::Null => {
                let scaler = Scalar::new(DataType::Null, AnyValue::Null);
                Ok(LiteralValue::Scalar(scaler))
            }
            v => Err(anyhow::anyhow!("Value {} is not supported", v)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dialect::TryDialect;

    use super::*;

    #[test]
    fn test_convert() {
        let url = "http://abc.com/a.csv";
        let sql = format!(
            "select a, b from {} where a > 10 order by a desc limit 10 offset 5",
            url
        );
        let statement =
            &sqlparser::parser::Parser::parse_sql(&TryDialect::default(), &sql).unwrap()[0];
        // println!("{:#?}", statement);
        let sql: Sql = statement
            .try_into()
            .map_err(|e| anyhow::anyhow!("{:#?}", e))
            .unwrap();
        println!("{:#?}", sql);
    }
}
