use pyo3::{
    Bound, PyResult, Python, pyfunction, pymodule,
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction,
};

#[pyfunction]
pub fn example_url() -> PyResult<String> {
    Ok(queryer::dialect::example_sql())
}

#[pyfunction]

pub fn query(sql: &str, output: Option<&str>) -> PyResult<String> {
    let rt = tokio::runtime::Runtime::new()?;
    let mut data = rt
        .block_on(async { queryer::query(sql).await })
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    match output {
        Some("csv") | None => Ok(data
            .to_csv()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?),
        _ => Err(pyo3::exceptions::PyValueError::new_err(
            "Only support csv output",
        )),
    }
}

#[pymodule]
fn queryer_py(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(example_url, m)?)?;
    m.add_function(wrap_pyfunction!(query, m)?)?;
    Ok(())
}
