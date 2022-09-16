use pyo3::prelude::*;
//use pyo3::exceptions::PyException;
use pyo3;


#[pyclass(module="pysqlx_core._core")]
#[derive(Debug, Clone)]
pub struct Test {
    #[pyo3(get, set)]
    name: String,
    #[pyo3(get, set)]
    age: i32,
    #[pyo3(get, set)]
    status: bool
}

#[pymethods]
impl Test {
    #[new]
    fn py_new(name: String, age: i32, status: bool) -> PyResult<Self>{
        Ok(Self {
            name,
            age,
            status
        })
    }
    fn __str__(&self) -> String {
        format!("Test(name={}, age={}, status={}", self.name, self.age, self.status)
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn pysqlx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<Test>()?;
    Ok(())
}