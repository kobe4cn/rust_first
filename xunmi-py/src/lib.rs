use ::xunmi as X;
use pyo3::{exceptions, prelude::*};
use std::fs;
use std::str::FromStr;

#[pyclass]
pub struct Indexer(X::Indexer);

#[pyclass]
pub struct IndexUpdater(X::IndexUpdater);

#[pyclass]
pub struct InputConfig(X::InputConfig);

#[pymodule]
fn xunmi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Indexer>()?;
    m.add_class::<IndexUpdater>()?;
    m.add_class::<InputConfig>()?;
    Ok(())
}

#[pymethods]
impl IndexUpdater {
    pub fn add(&mut self, text: &str, config: &InputConfig) -> PyResult<()> {
        Ok(self.0.add(text, &config.0).map_err(to_py_err)?)
    }

    pub fn update(&mut self, text: &str, config: &InputConfig) -> PyResult<()> {
        Ok(self.0.update(text, &config.0).map_err(to_py_err)?)
    }

    pub fn commit(&self) -> PyResult<()> {
        Ok(self.0.commit().map_err(to_py_err)?)
    }

    pub fn clear(&self) -> PyResult<()> {
        Ok(self.0.clear().map_err(to_py_err)?)
    }
}

#[pymethods]
impl Indexer {
    #[new]
    pub fn open_or_create(filename: &str) -> PyResult<Indexer> {
        let cotent = fs::read_to_string(filename).map_err(to_py_err)?;
        let config = X::IndexConfig::from_str(&cotent).map_err(to_py_err)?;
        let indexer = X::Indexer::open_or_create(config).map_err(to_py_err)?;
        Ok(Indexer(indexer))
    }

    pub fn get_updater(&self) -> IndexUpdater {
        IndexUpdater(self.0.get_updater())
    }
    pub fn search(
        &self,
        query: String,
        fields: Vec<String>,
        limit: usize,
        offset: usize,
    ) -> PyResult<Vec<(f32, String)>> {
        let default_fields = fields.iter().map(|f| f.as_str()).collect::<Vec<_>>();
        let results = self
            .0
            .search(&query, &default_fields, limit, offset)
            .map_err(to_py_err)?
            .into_iter()
            .map(|(score, doc)| (score, serde_json::to_string(&doc).unwrap()))
            .collect::<Vec<_>>();
        Ok(results)
    }

    pub fn reload(&self) -> PyResult<()> {
        Ok(self.0.reload().map_err(to_py_err)?)
    }
}

pub(crate) fn to_py_err<E: ToString>(e: E) -> PyErr {
    exceptions::PyValueError::new_err(e.to_string())
}

#[pymethods]
impl InputConfig {
    #[new]
    fn new(
        input_type: &str,
        mapping: Option<Vec<(String, String)>>,
        conversion: Option<Vec<(String, (String, String))>>,
    ) -> PyResult<InputConfig> {
        let input_type = match input_type {
            "json" => X::InputType::Json,
            "yaml" => X::InputType::Yaml,
            "xml" => X::InputType::Xml,
            _ => return Err(to_py_err(format!("invalid input type: {}", input_type))),
        };
        let conversion = conversion
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(k, (v1, v2))| {
                let t = match (v1.as_ref(), v2.as_ref()) {
                    ("string", "number") => (X::ValueType::String, X::ValueType::Number),
                    ("number", "string") => (X::ValueType::Number, X::ValueType::String),
                    _ => return None,
                };
                Some((k, t))
            })
            .collect::<Vec<_>>();
        Ok(InputConfig(X::InputConfig::new(
            input_type,
            mapping.unwrap_or_default(),
            conversion,
        )))
    }
}
