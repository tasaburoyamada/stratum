use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use ::stratum::storage::storage_context::StorageContext;

/// A thin wrapper around the Stratum Rust core.
#[pyclass(unsendable)]
struct Stratum {
    // We hold the context. In a real scenario, you'd also hold the retriever/llm etc.
    _context: StorageContext,
}

#[pymethods]
impl Stratum {
    #[new]
    fn new(persist_dir: Option<String>) -> PyResult<Self> {
        let context = if let Some(dir) = persist_dir {
            StorageContext::from_dir(&dir).map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        } else {
            StorageContext::in_memory()
        };

        Ok(Self { _context: context })
    }

    /// Basic health check/ping method.
    fn ping(&self) -> PyResult<String> {
        Ok("Stratum Core (Rust) is active.".to_string())
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn stratum(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Stratum>()?;
    Ok(())
}
