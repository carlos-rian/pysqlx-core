use convert::convert_result_set;
use convert::convert_result_set_as_list;
use py_types::PyRow;
use py_types::PyRows;
use py_types::{py_error, DBError, PySQLXError, PySQLXResult};
use pyo3::prelude::*;
use quaint::connector::IsolationLevel;
use quaint::prelude::*;
use quaint::single::Quaint;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Connection {
    conn: Quaint,
}

impl Connection {
    // create a new connection using the given url
    pub async fn new(uri: String) -> Result<Self, PySQLXError> {
        let conn = match Quaint::new(uri.as_str()).await {
            Ok(r) => r,
            Err(e) => return Err(py_error(e, DBError::ConnectError)),
        };
        Ok(Self { conn })
    }

    // Execute a query given as SQL, interpolating the given parameters. return a PySQLXResult
    async fn _query(&self, sql: &str) -> Result<PySQLXResult, PySQLXError> {
        match self.conn.query_raw(sql, &[]).await {
            Ok(r) => Ok(convert_result_set(r)),
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }
    // Execute a query given as SQL, interpolating the given parameters. return a list of rows
    async fn _query_as_list(&self, sql: &str) -> Result<PyRows, PySQLXError> {
        match self.conn.query_raw(sql, &[]).await {
            Ok(r) => Ok(convert_result_set_as_list(r)),
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }
    // Execute a query given as SQL, interpolating the given parameters. return a dict of rows
    async fn _query_first_as_dict(&self, sql: &str) -> Result<PyRow, PySQLXError> {
        match self.conn.query_raw(sql, &[]).await {
            Ok(r) => {
                let rows = convert_result_set_as_list(r);
                match rows.get(0) {
                    Some(r) => Ok(r.clone()),
                    None => Ok(PyRow::new()),
                }
            }
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }
    // Execute a query given as SQL, interpolating the given parameters and returning the number of affected rows.
    async fn _execute(&self, sql: &str) -> Result<u64, PySQLXError> {
        match self.conn.execute_raw(sql, &[]).await {
            Ok(r) => Ok(r),
            Err(e) => Err(py_error(e, DBError::ExecuteError)),
        }
    }
    // Run a command in the database, for queries that can't be run using prepared statements.
    async fn _raw_cmd(&self, sql: &str) -> Result<(), PySQLXError> {
        match self.conn.raw_cmd(sql).await {
            Ok(_) => Ok(()),
            Err(e) => Err(py_error(e, DBError::RawCmdError)),
        }
    }
    // return the isolation level
    fn get_isolation_level(&self, isolation_level: String) -> Result<IsolationLevel, PySQLXError> {
        match isolation_level.to_uppercase().as_str() {
            "READUNCOMMITTED" => Ok(IsolationLevel::ReadUncommitted),
            "READCOMMITTED" => Ok(IsolationLevel::ReadCommitted),
            "REPEATABLEREAD" => Ok(IsolationLevel::RepeatableRead),
            "SNAPSHOT" => Ok(IsolationLevel::Snapshot),
            "SERIALIZABLE" => Ok(IsolationLevel::Serializable),
            _ => {
                return Err(PySQLXError::new(
                    "PY001IL".to_string(),
                    "invalid isolation level".to_string(),
                    DBError::IsoLevelError,
                ))
            }
        }
    }

    // Sets the transaction isolation level to given value. Implementers have to make sure that the passed isolation level is valid for the underlying database.
    async fn _set_isolation_level(&self, isolation_level: String) -> Result<(), PySQLXError> {
        let level = self.get_isolation_level(isolation_level)?;
        match self.conn.set_tx_isolation_level(level).await {
            Ok(_) => Ok(()),
            Err(e) => Err(py_error(e, DBError::IsoLevelError)),
        }
    }

    // Start a new transaction.
    async fn _start_transaction(&self, isolation_level: Option<String>) -> Result<(), PySQLXError> {
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

#[pymethods]
impl Connection {
    pub fn query<'a>(&self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._query(sql.as_str()).await {
                Ok(r) => Ok(r),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }

    pub fn execute<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._execute(sql.as_str()).await {
                Ok(r) => Python::with_gil(|py| Ok(r.to_object(py))),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }

    pub fn query_as_list<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let rows = match slf._query_as_list(sql.as_str()).await {
                Ok(r) => r,
                Err(e) => return Err(e.to_pyerr()),
            };
            Python::with_gil(|py| {
                let pyrows = rows.to_object(py);
                Ok(pyrows)
            })
        })
    }

    pub fn query_first_as_dict<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let row = match slf._query_first_as_dict(sql.as_str()).await {
                Ok(r) => r,
                Err(e) => return Err(e.to_pyerr()),
            };
            Python::with_gil(|py| {
                let pyrow = row.to_object(py);
                Ok(pyrow)
            })
        })
    }

    pub fn is_healthy(&self) -> bool {
        self.conn.is_healthy()
    }

    pub fn requires_isolation_first(&self) -> bool {
        self.conn.requires_isolation_first()
    }

    pub fn raw_cmd<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._raw_cmd(sql.as_str()).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }

    pub fn set_isolation_level<'a>(
        &mut self,
        py: Python<'a>,
        isolation_level: String,
    ) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._set_isolation_level(isolation_level).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }

    pub fn start_transaction<'a>(
        &mut self,
        py: Python<'a>,
        isolation_level: Option<String>,
    ) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._start_transaction(isolation_level).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use py_types::PyValue;

    use super::*;

    #[tokio::test]
    async fn test_connection_query_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._query("SELECT 1 as number").await.unwrap();
        assert_eq!(res.rows().len(), 1);
        assert_eq!(res.types().len(), 1);
        assert_eq!(res.types().get("number").unwrap(), "int");
    }

    #[tokio::test]
    async fn test_col_without_name_query_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();

        let res = conn._query("SELECT 1, 2").await.unwrap();

        assert_eq!(
            res.rows().get(0).unwrap().get("col_1").unwrap().clone(),
            PyValue::Int(1)
        );
        assert_eq!(
            res.rows().get(0).unwrap().get("col_2").unwrap().clone(),
            PyValue::Int(2)
        );

        assert_eq!(res.types().get("col_1").unwrap(), "int");
        assert_eq!(res.types().get("col_2").unwrap(), "int");

        let res = conn._query("SELECT -1.3, -453.32").await.unwrap();

        assert_eq!(
            res.rows().get(0).unwrap().get("col_1_3").unwrap().clone(),
            PyValue::Float(-1.3)
        );

        assert_eq!(
            res.rows()
                .get(0)
                .unwrap()
                .get("col_453_32")
                .unwrap()
                .clone(),
            PyValue::Float(-453.32)
        );
        assert_eq!(res.types().get("col_1_3").unwrap(), "float");
        assert_eq!(res.types().get("col_453_32").unwrap(), "float");
    }

    #[tokio::test]
    async fn test_connection_query_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._query("SELECT * FROM InvalidTable").await;
        assert!(res.is_err())
    }

    #[tokio::test]
    async fn test_connection_execute_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._execute("CREATE TABLE IF NOT EXISTS test (id int)")
            .await
            .unwrap();
        assert_eq!(res, 0);

        let res = conn
            ._execute("INSERT INTO test (id) VALUES (1)")
            .await
            .unwrap();
        assert_eq!(res, 1);
    }

    #[tokio::test]
    async fn test_connection_execute_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._execute("CREATE TABL test (id int)").await;
        assert!(res.is_err())
    }

    #[tokio::test]
    async fn test_query_as_list_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._execute("CREATE TABLE IF NOT EXISTS test (id int)")
            .await
            .unwrap();
        assert_eq!(res, 0);

        let res = conn
            ._execute("INSERT INTO test (id) VALUES (1)")
            .await
            .unwrap();
        assert_eq!(res, 1);

        let res = conn._query_as_list("SELECT * FROM test").await.unwrap();
        assert_eq!(res[0].get("id").unwrap(), &PyValue::Int(1));
    }

    #[tokio::test]
    async fn test_query_as_list_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn._query_as_list("SELECT * FROM InvalidTable").await;
        assert!(res.is_err())
    }

    #[tokio::test]
    async fn test_query_first_as_dict_success() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._execute("CREATE TABLE IF NOT EXISTS test (id int)")
            .await
            .unwrap();
        assert_eq!(res, 0);

        let res = conn
            ._execute("INSERT INTO test (id) VALUES (1)")
            .await
            .unwrap();
        assert_eq!(res, 1);

        let res = conn
            ._query_first_as_dict("SELECT * FROM test")
            .await
            .unwrap();
        assert_eq!(res.get("id").unwrap(), &PyValue::Int(1));
    }

    #[tokio::test]
    async fn test_query_first_as_dict_success_empty() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._execute("CREATE TABLE IF NOT EXISTS test (id int)")
            .await
            .unwrap();
        assert_eq!(res, 0);

        let res = conn
            ._query_first_as_dict("SELECT * FROM test WHERE id = 0")
            .await
            .unwrap();
        assert_eq!(res.len(), 0);
    }

    #[tokio::test]
    async fn test_query_first_as_dict_error() {
        let conn = Connection::new("file:///tmp/db.db".to_string())
            .await
            .unwrap();
        let res = conn
            ._query_first_as_dict("SELECT * FROM InvalidTable")
            .await;
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
