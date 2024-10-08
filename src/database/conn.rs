use crate::convert::convert_result_set;
use crate::convert::convert_result_set_as_list;
use crate::py_types::{py_error, DBError, PySQLxError, PySQLxResponse, PySQLxStatement};
use crate::py_types::{PySQLxRow, PySQLxRows};
use pyo3::prelude::*;
use quaint::connector::IsolationLevel;
use quaint::prelude::*;
use quaint::single::Quaint;

use crate::tokio_runtime;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Connection {
    conn: Quaint,
}

pub type PySQLxResult<T> = Result<T, PySQLxError>;

impl Connection {
    // create a new connection using the given url
    pub async fn new(uri: String) -> PySQLxResult<Self> {
        let conn = match Quaint::new(uri.as_str()).await {
            Ok(r) => r,
            Err(e) => return Err(py_error(e, DBError::ConnectError)),
        };
        Ok(Self { conn })
    }

    // Execute a query given as SQL, interpolating the given parameters. return a PySQLXResult
    async fn _query_typed(&self, sql: &str, params: &[Value<'_>]) -> PySQLxResult<PySQLxResponse> {
        match self.conn.query_raw(sql, params).await {
            Ok(r) => Ok(convert_result_set(r)),
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }
    // Execute a query given as SQL, interpolating the given parameters. return a list of rows
    async fn _query_all(&self, sql: &str, params: &[Value<'_>]) -> PySQLxResult<PySQLxRows> {
        match self.conn.query_raw(sql, params).await {
            Ok(r) => Ok(convert_result_set_as_list(r)),
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }
    // Execute a query given as SQL, interpolating the given parameters. return a dict of rows
    async fn _query_one(&self, sql: &str, params: &[Value<'_>]) -> PySQLxResult<PySQLxRow> {
        match self.conn.query_raw(sql, params).await {
            Ok(r) => {
                let rows = convert_result_set_as_list(r);
                match rows.get(0) {
                    Some(r) => Ok(r.clone()),
                    None => Ok(PySQLxRow::new()),
                }
            }
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }
    // Execute a query given as SQL, interpolating the given parameters and returning the number of affected rows.
    async fn _execute(&self, sql: &str, params: &[Value<'_>]) -> PySQLxResult<u64> {
        match self.conn.execute_raw(sql, params).await {
            Ok(r) => Ok(r),
            Err(e) => Err(py_error(e, DBError::ExecuteError)),
        }
    }
    // Run a command in the database, for queries that can't be run using prepared statements.
    async fn _raw_cmd(&self, sql: &str) -> PySQLxResult<()> {
        match self.conn.raw_cmd(sql).await {
            Ok(_) => Ok(()),
            Err(e) => Err(py_error(e, DBError::RawCmdError)),
        }
    }
    // return the isolation level
    fn get_isolation_level(&self, isolation_level: String) -> PySQLxResult<IsolationLevel> {
        match isolation_level.to_uppercase().as_str() {
            "READUNCOMMITTED" => Ok(IsolationLevel::ReadUncommitted),
            "READCOMMITTED" => Ok(IsolationLevel::ReadCommitted),
            "REPEATABLEREAD" => Ok(IsolationLevel::RepeatableRead),
            "SNAPSHOT" => Ok(IsolationLevel::Snapshot),
            "SERIALIZABLE" => Ok(IsolationLevel::Serializable),
            _ => {
                return Err(PySQLxError::py_new(
                    "PY001IL".to_string(),
                    "invalid isolation level".to_string(),
                    DBError::IsoLevelError,
                ))
            }
        }
    }

    // Sets the transaction isolation level to given value. Implementers have to make sure that the passed isolation level is valid for the underlying database.
    async fn _set_isolation_level(&self, isolation_level: String) -> PySQLxResult<()> {
        let level = self.get_isolation_level(isolation_level)?;
        match self.conn.set_tx_isolation_level(level).await {
            Ok(_) => Ok(()),
            Err(e) => Err(py_error(e, DBError::IsoLevelError)),
        }
    }

    // Start a new transaction.
    async fn _start_transaction(&self, isolation_level: Option<String>) -> PySQLxResult<()> {
        let level = match isolation_level {
            Some(l) => Some(self.get_isolation_level(l)?),
            None => None,
        };

        match self.conn.start_transaction(level).await {
            Ok(_) => Ok(()),
            Err(e) => Err(py_error(e, DBError::StartTransactionError)),
        }
    }
}

// default methods
#[pymethods]
impl Connection {
    pub fn is_healthy(&self) -> bool {
        self.conn.is_healthy()
    }

    pub fn requires_isolation_first(&self) -> bool {
        self.conn.requires_isolation_first()
    }
    // async methods
    #[pyo3(signature=(stmt))]
    pub async fn query_typed(&self, stmt: PySQLxStatement) -> PyResult<PySQLxResponse> {
        let slf = self.clone();
        tokio_runtime()
            .spawn(async move {
                let (sql, params) = stmt.prepared_sql();
                let res = match slf._query_typed(sql.as_str(), params.as_slice()).await {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_pyerr()),
                };

                Python::with_gil(|_py| Ok(res))
            })
            .await
            .unwrap()
    }

    #[pyo3(signature=(stmt))]
    pub async fn execute(&self, stmt: PySQLxStatement) -> PyResult<Py<PyAny>> {
        let slf = self.clone();
        tokio_runtime()
            .spawn(async move {
                let (sql, params) = stmt.prepared_sql();
                let res = match slf._execute(sql.as_str(), params.as_slice()).await {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_pyerr()),
                };

                Python::with_gil(|py| Ok(res.to_object(py)))
            })
            .await
            .unwrap()
    }

    #[pyo3(signature=(stmt))]
    pub async fn query_all(&self, stmt: PySQLxStatement) -> PyResult<Py<PyAny>> {
        let slf = self.clone();
        tokio_runtime()
            .spawn(async move {
                let (sql, p) = stmt.prepared_sql();
                let res = match slf._query_all(sql.as_str(), p.as_slice()).await {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_pyerr()),
                };

                Python::with_gil(|py| Ok(res.to_object(py)))
            })
            .await
            .unwrap()
    }

    #[pyo3(signature=(stmt))]
    pub async fn query_one(&self, stmt: PySQLxStatement) -> PyResult<Py<PyAny>> {
        let slf = self.clone();
        tokio_runtime()
            .spawn(async move {
                let (sql, params) = (stmt.get_sql(), stmt.get_params());
                let res = match slf._query_one(sql.as_str(), params.as_slice()).await {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_pyerr()),
                };

                Python::with_gil(|py| Ok(res.to_object(py)))
            })
            .await
            .unwrap()
    }

    #[pyo3(signature=(stmt))]
    pub async fn raw_cmd(&self, stmt: PySQLxStatement) -> PyResult<Py<PyAny>> {
        let slf = self.clone();
        tokio_runtime()
            .spawn(async move {
                let (sql, _) = (stmt.get_sql(), stmt.get_params());
                match slf._raw_cmd(sql.as_str()).await {
                    Ok(_) => Python::with_gil(|py| Ok(py.None())),
                    Err(e) => Err(e.to_pyerr()),
                }
            })
            .await
            .unwrap()
    }

    #[pyo3(signature=(isolation_level))]
    pub async fn set_isolation_level(&self, isolation_level: String) -> PyResult<Py<PyAny>> {
        let slf = self.clone();
        tokio_runtime()
            .spawn(async move {
                match slf._set_isolation_level(isolation_level).await {
                    Ok(_) => Python::with_gil(|py| Ok(py.None())),
                    Err(e) => Err(e.to_pyerr()),
                }
            })
            .await
            .unwrap()
    }

    #[pyo3(signature=(isolation_level = None))]
    pub async fn start_transaction(&self, isolation_level: Option<String>) -> PyResult<Py<PyAny>> {
        let slf = self.clone();
        tokio_runtime()
            .spawn(async move {
                match slf._start_transaction(isolation_level).await {
                    Ok(_) => Python::with_gil(|py| Ok(py.None())),
                    Err(e) => Err(e.to_pyerr()),
                }
            })
            .await
            .unwrap()
    }

    // sync methods
    #[pyo3(signature=(stmt))]
    pub fn query_typed_sync(&self, stmt: PySQLxStatement) -> PyResult<PySQLxResponse> {
        let slf = self.clone();
        let (sql, params) = stmt.prepared_sql();
        let res = tokio_runtime().block_on(async move {
            match slf._query_typed(sql.as_str(), params.as_slice()).await {
                Ok(r) => Ok(r),
                Err(e) => Err(e),
            }
        });

        match res {
            Ok(r) => Ok(r),
            Err(e) => Err(e.to_pyerr()),
        }
    }

    #[pyo3(signature=(stmt))]
    pub fn execute_sync(&self, stmt: PySQLxStatement) -> PyResult<Py<PyAny>> {
        let slf = self.clone();
        let (sql, params) = stmt.prepared_sql();
        let res = tokio_runtime().block_on(async move {
            match slf._execute(sql.as_str(), params.as_slice()).await {
                Ok(r) => Ok(r),
                Err(e) => Err(e),
            }
        });

        match res {
            Ok(r) => Python::with_gil(|py| Ok(r.to_object(py))),
            Err(e) => Err(e.to_pyerr()),
        }
    }

    #[pyo3(signature=(stmt))]
    pub fn query_all_sync(&self, stmt: PySQLxStatement) -> PyResult<Py<PyAny>> {
        let slf = self.clone();
        let (sql, p) = stmt.prepared_sql();
        let res = tokio_runtime().block_on(async move {
            match slf._query_all(sql.as_str(), p.as_slice()).await {
                Ok(r) => Ok(r),
                Err(e) => Err(e),
            }
        });

        match res {
            Ok(r) => Python::with_gil(|py| Ok(r.to_object(py))),
            Err(e) => Err(e.to_pyerr()),
        }
    }

    #[pyo3(signature=(stmt))]
    pub fn query_one_sync(&self, stmt: PySQLxStatement) -> PyResult<Py<PyAny>> {
        let slf = self.clone();
        let (sql, params) = (stmt.get_sql(), stmt.get_params());
        let res = tokio_runtime().block_on(async move {
            match slf._query_one(sql.as_str(), params.as_slice()).await {
                Ok(r) => Ok(r),
                Err(e) => Err(e),
            }
        });

        match res {
            Ok(r) => Python::with_gil(|py| Ok(r.to_object(py))),
            Err(e) => Err(e.to_pyerr()),
        }
    }

    #[pyo3(signature=(stmt))]
    pub fn raw_cmd_sync(&self, stmt: PySQLxStatement) -> PyResult<()> {
        let slf = self.clone();
        let (sql, _) = (stmt.get_sql(), stmt.get_params());
        let res = tokio_runtime().block_on(async move {
            match slf._raw_cmd(sql.as_str()).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        });

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_pyerr()),
        }
    }

    #[pyo3(signature=(isolation_level))]
    pub fn set_isolation_level_sync(&self, isolation_level: String) -> PyResult<()> {
        let slf = self.clone();
        let res = tokio_runtime().block_on(async move {
            match slf._set_isolation_level(isolation_level).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        });

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_pyerr()),
        }
    }

    #[pyo3(signature=(isolation_level = None))]
    pub fn start_transaction_sync(&self, isolation_level: Option<String>) -> PyResult<()> {
        let slf = self.clone();
        let res = tokio_runtime().block_on(async move {
            match slf._start_transaction(isolation_level).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        });

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_pyerr()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::py_types::PySQLxValue;

    use super::*;

    #[tokio::test]
    async fn test_connection_query_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._query_typed("SELECT ? as number", &[Value::from(1)])
            .await
            .unwrap();
        assert_eq!(res.rows().len(), 1);
        assert_eq!(res.types().len(), 1);
        assert_eq!(res.types().get("number").unwrap(), "int");
    }

    #[tokio::test]
    async fn test_col_without_name_query_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();

        let res = conn
            ._query_typed("SELECT ?, ?", &[Value::from(1), Value::from(2)])
            .await
            .unwrap();

        assert_eq!(
            res.rows().get(0).unwrap().get("col_0").unwrap().clone(),
            PySQLxValue::Int(1)
        );
        assert_eq!(
            res.rows().get(0).unwrap().get("col_1").unwrap().clone(),
            PySQLxValue::Int(2)
        );

        assert_eq!(res.types().get("col_0").unwrap(), "int");
        assert_eq!(res.types().get("col_1").unwrap(), "int");

        let res = conn
            ._query_typed("SELECT -1.3, -453.32", &[])
            .await
            .unwrap();

        assert_eq!(
            res.rows().get(0).unwrap().get("col_1_3").unwrap().clone(),
            PySQLxValue::Float(-1.3)
        );

        assert_eq!(
            res.rows()
                .get(0)
                .unwrap()
                .get("col_453_32")
                .unwrap()
                .clone(),
            PySQLxValue::Float(-453.32)
        );
        assert_eq!(res.types().get("col_1_3").unwrap(), "float");
        assert_eq!(res.types().get("col_453_32").unwrap(), "float");
    }

    #[tokio::test]
    async fn test_connection_query_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._query_typed("SELECT * FROM InvalidTable", &[]).await;
        assert!(res.is_err())
    }

    #[tokio::test]
    async fn test_connection_execute_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._execute("CREATE TABLE IF NOT EXISTS test (id int)", &[])
            .await
            .unwrap();
        assert_eq!(res, 0);

        let res = conn
            ._execute("INSERT INTO test (id) VALUES (?)", &[Value::from(1)])
            .await
            .unwrap();
        assert_eq!(res, 1);
    }

    #[tokio::test]
    async fn test_connection_execute_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._execute("CREATE TABL test (id int)", &[]).await;
        assert!(res.is_err())
    }

    #[tokio::test]
    async fn test_query_as_list_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._execute("CREATE TABLE IF NOT EXISTS test (id int)", &[])
            .await
            .unwrap();
        assert_eq!(res, 0);

        let res = conn
            ._execute("INSERT INTO test (id) VALUES (?)", &[Value::from(1)])
            .await
            .unwrap();
        assert_eq!(res, 1);

        let res = conn._query_all("SELECT * FROM test", &[]).await.unwrap();
        assert_eq!(res[0].get("id").unwrap(), &PySQLxValue::Int(1));
    }

    #[tokio::test]
    async fn test_query_as_list_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._query_all("SELECT * FROM InvalidTable", &[]).await;
        assert!(res.is_err())
    }

    #[tokio::test]
    async fn test_query_first_as_dict_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._execute("CREATE TABLE IF NOT EXISTS test (id int)", &[])
            .await
            .unwrap();
        assert_eq!(res, 0);

        let res = conn
            ._execute("INSERT INTO test (id) VALUES (?)", &[Value::from(1)])
            .await
            .unwrap();
        assert_eq!(res, 1);

        let res = conn._query_one("SELECT * FROM test", &[]).await.unwrap();
        assert_eq!(res.get("id").unwrap(), &PySQLxValue::Int(1));
    }

    #[tokio::test]
    async fn test_query_first_as_dict_success_empty() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._execute("CREATE TABLE IF NOT EXISTS test (id int)", &[])
            .await
            .unwrap();
        assert_eq!(res, 0);

        let res = conn
            ._query_one("SELECT * FROM test WHERE id = ?", &[Value::from(0)])
            .await
            .unwrap();
        assert_eq!(res.len(), 0);
    }

    #[tokio::test]
    async fn test_query_first_as_dict_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._query_one("SELECT * FROM InvalidTable", &[]).await;
        assert!(res.is_err())
    }

    #[tokio::test]
    async fn test_raw_cmd_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._raw_cmd("CREATE TABLE IF NOT EXISTS test (id int)")
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_raw_cmd_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._raw_cmd("CREATE TABL test (id int)").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_set_isolation_level_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._set_isolation_level("InvalidRead".to_string()).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_start_transaction_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._start_transaction(None).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_start_transaction_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._start_transaction(Some("InvalidRead".to_string()))
            .await;
        assert!(res.is_err());
    }
}
