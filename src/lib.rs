pub mod base;
pub mod db;
mod pyvalue;
pub mod record;
pub mod test_conn;
pub mod value;

use base::error::PysqlxDBError;
//use base::row::PysqlxValue;
//use std::collections::HashMap;
use db::PyConnection;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pythonize::pythonize;
use test_conn::Connection;

#[pyfunction]
pub fn connect<'a>(py: Python<'a>, uri: String) -> Result<&'a PyAny, pyo3::PyErr> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let conn = match PyConnection::new(uri).await {
            Ok(r) => r,
            Err(e) => {
                return Err(PyErr::from(e));
            }
        };
        Python::with_gil(|py| Ok(conn.into_py(py)))
    })
}

#[pyfunction]
pub fn query<'a>(py: Python<'a>, conn: Py<PyAny>, sql: String) -> Result<&'a PyAny, pyo3::PyErr> {
    let db = conn.extract::<PyConnection>(py)?;
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let rows = match db.query(sql).await {
            Ok(r) => r,
            Err(e) => {
                return Err(PyErr::from(e));
            }
        };
        Python::with_gil(|py| Ok(rows.into_py(py)))
    })
}

async fn test() -> Py<PyAny> {
    let uri = "postgresql://postgres:password@localhost:5432/fastapi_prisma?schema=public";
    let sql = "select * from peoples;";

    match Connection::new(uri.to_string()).await {
        Ok(conn) => match conn.query(sql).await {
            Ok(rows) => {
                let row = rows.rows();
                //let gil = Python::acquire_gil();
                //let py = gil.python();
                Python::with_gil(|py| {
                    let res = pythonize(py, &row).unwrap();
                    res
                })
            }
            Err(e) => {
                panic!("{:?}", PysqlxDBError::from(e));
            }
        },
        Err(e) => {
            panic!("{:?}", PysqlxDBError::from(e));
        }
    }
}

#[pyfunction]
fn test_query<'a>(py: Python<'a>) -> PyResult<&'a PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let rows = test().await;
        Ok(rows)
    })
}

#[pymodule]
fn pysqlx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add_function(wrap_pyfunction!(query, m)?)?;
    m.add_function(wrap_pyfunction!(test_query, m)?)?;
    m.add_class::<PysqlxDBError>()?;
    Ok(())
}
