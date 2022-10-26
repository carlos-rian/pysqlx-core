from typing import Any, Dict, List, Union
from typing_extensions import Literal

__all__ = ("__version__", "new", "Connection", 'PySQLXError', "PySQLXResult")
__version__: str

IsolationLevel = Literal["ReadUncommitted", "ReadCommitted", "RepeatableRead", "Snapshot", "Serializable"]

class PySQLXError(Exception):
    """
    ## PySQLXError

    ### Base class for all exceptions raised by pysqlx.

    `code`: str - mapped to the error code
    `message`: str - information about the error
    `error`: str - type of error

    error types:
        * QueryError
        * ExecuteError
        * ConnectionError
        * IsoLevelError
        * StartTransactionError
    """

    code: str
    message: str
    error: str

    def code(self) -> str:
        """Returns the error code"""
        ...
    def message(self) -> str:
        """Returns the error message"""
        ...
    def error(self) -> str:
        """
        Returns the type of the error
            * QueryError
            * ExecuteError
            * ConnectionError
            * IsoLevelError
            * StartTransactionError

        """
        ...

class PySQLXResult:
    """
    PySQLXResult is a class that represents the result of a query.
    It is returned by the `query` method of the `Connection` class.
    """

    def get_types(self) -> "Dict[str, str]":
        """Returns a dictionary of column names and their types used to generate Pydantic BaseModel."""
        raise PySQLXError()
    def get_all(self) -> "List[Dict[str, Any]]":
        """Returns a list of dictionaries representing the rows of the query result."""
        raise PySQLXError()
    def get_first(self) -> "Dict[str, Any]":
        """Returns the first row of the query result as a dictionary."""
        raise PySQLXError()
    def __len__(self) -> int:
        """Returns the number of rows in the query result."""
        ...

class Connection:
    """
    ## Connection
    Creates a new connection to the database. Create after calling `new`.

    ### example
    ``` python
    import pysqlx_core
    import asyncio

    async def main(sql):
        # Create a connection
        db = await pysqlx_core.new(uri="file:///tmp/db.db")

        # Create a table
        await db.execute(sql='''
            CREATE TABLE IF NOT EXISTS test (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
            '''
        )

        # Insert a row and return quantity rows affected
        await db.execute(sql="INSERT INTO test (name) VALUES ('Carlos');")

        # Select all rows, return a class PySQLXResult
        result = await db.query(sql="SELECT * FROM test;")
        # get first row
        row = result.get_first() # Dict[str, Any]
        # get all rows
        rows = result.get_all() # List[Dict[str, Any]]
        #return the db types to Pydantic BaseModel
        types = result.get_model() # Dict[str, str]

        # Select all rows, return how List[Dict[str, Any]]
        rows = await db.query_as_list(sql="SELECT * FROM test;")

        # close? no need 👌-> auto-close when finished programmer or go out of context..

    asyncio.run(main())
    ```
    """

    async def query(self, sql: str) -> "PySQLXResult":
        """Returns a `PySQLXResult` object representing the result of the query."""
        raise PySQLXError()
    async def execute(self, sql: str) -> "int":
        """Executes a query and returns the number of rows affected."""
        raise PySQLXError()
    async def query_as_list(self, sql: str) -> "List[Dict[str, Any]]":
        """Returns a list of dictionaries representing the rows of the query result."""
        raise PySQLXError()
    async def query_first_as_dict(self, sql: str) -> "Dict[str, Any]":
        """Returns the first row of the query result as a dictionary."""
        raise PySQLXError()
    async def raw_cmd(self, sql: str) -> "None":
        """Run a command in the database, for queries that can't be run using prepared statements."""
        raise PySQLXError()
    def is_healthy(self) -> "bool":
        """Returns false, if connection is considered to not be in a working state"""
        ...
    def requires_isolation_first(self) -> "bool":
        """Returns `True` if the connection requires isolation first, `False` otherwise.
        This is used to determine if the connection should be isolated before executing a query.
        for example, sqlserver requires isolation before executing a query using begin.

        - Signals if the isolation level SET needs to happen before or after the BEGIN

        * [SQL Server documentation]: (https://docs.microsoft.com/en-us/sql/t-sql/statements/set-transaction-isolation-level-transact-sql?view=sql-server-ver15)
        * [Postgres documentation]: (https://www.postgresql.org/docs/current/sql-set-transaction.html)
        * [MySQL documentation]: (https://dev.mysql.com/doc/refman/8.0/en/innodb-transaction-isolation-levels.html)
        * [SQLite documentation]: (https://www.sqlite.org/isolation.html)
        """
        ...
    async def set_isolation_level(self, isolation_level: "IsolationLevel") -> "None":
        """
        Sets the isolation level of the connection.
        The isolation level is set before the transaction is started.
        Is used to separate the transaction per level.

        * [SQL Server documentation]: (https://docs.microsoft.com/en-us/sql/t-sql/statements/set-transaction-isolation-level-transact-sql?view=sql-server-ver15)
        * [Postgres documentation]: (https://www.postgresql.org/docs/current/sql-set-transaction.html)
        * [MySQL documentation]: (https://dev.mysql.com/doc/refman/8.0/en/innodb-transaction-isolation-levels.html)
        * [SQLite documentation]: (https://www.sqlite.org/isolation.html)
        """
        raise PySQLXError()
    async def start_transaction(self, isolation_level: "Union[IsolationLevel, None]") -> "None":
        """Starts a transaction with BEGIN. by default, does not set the isolation level."""
        raise PySQLXError()

async def new(uri: str) -> 'Connection':
    """Creates a new connection to the database. Returns a `Connection` object."""
    raise PySQLXError()
