from datetime import date, datetime, time
from decimal import Decimal
from enum import Enum
from typing import Any, Dict, List, Self, Union
import sys
from uuid import UUID

if sys.version_info >= (3, 8):
    from typing import Literal
else:
    from typing_extensions import Literal

__all__ = ("__version__", "new", "Connection", "PySQLxError", "PySQLxResult")
__version__: str

IsolationLevel = Literal[
    "ReadUncommitted", "ReadCommitted", "RepeatableRead", "Snapshot", "Serializable"
]

class EnumArray(tuple): ...

SupportedValueType = Union[
    bool,
    str,
    int,
    Dict[str, Any],
    List[Dict[str, Any]],
    UUID,
    time,
    date,
    datetime,
    float,
    bytes,
    Decimal,
    Enum,
    None,
]

class PySQLxError(Exception):
    """
    ## PySQLxError

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

class PySQLxInvalidParamError(Exception):
    typ_from: str
    typ_to: str
    details: str

    def typ_from(self) -> str:
        """
        Return the typ_from of the error
        """
        ...

    def typ_to(self) -> str:
        """
        Return the typ_to of the error
        """
        ...

    def details(self) -> str:
        """
        Return the details of the error
        """
        ...

class PySQLxStatement:
    """
    ## PySQLxStatement

    Represents a prepared statement. The class prepares the statement and binds the parameters to use in the Connection class.

    """

    def __init__(
        self,
        provider: str,
        sql: str,
        params: Union[Dict[str, SupportedValueType], None],
    ) -> Self: ...

class PySQLxResult:
    """
    PySQLxResult is a class that represents the result of a query.
    It is returned by the `query` method of the `Connection` class.
    """

    def get_types(self) -> "Dict[str, str]":
        """Returns a dictionary of column names and their types used to generate Pydantic BaseModel."""
        raise PySQLxError()
    def get_all(self) -> "List[Dict[str, Any]]":
        """Returns a list of dictionaries representing the rows of the query result."""
        raise PySQLxError()
    def get_first(self) -> "Dict[str, Any]":
        """Returns the first row of the query result as a dictionary."""
        raise PySQLxError()
    def get_last_insert_id(self) -> "Union[int, None]":
        """Returns the last inserted id."""
        ...

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

        # Select all rows, return a class PySQLxResult
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

    async def query(self, sql: str, params: Union[tuple, None]) -> "PySQLxResult":
        """Returns a `PySQLxResult` object representing the result of the query."""
        raise PySQLxError()
    async def execute(self, sql: str, params: Union[tuple, None]) -> "int":
        """Executes a query and returns the number of rows affected."""
        raise PySQLxError()
    async def query_as_list(
        self, sql: str, params: Union[tuple, None]
    ) -> "List[Dict[str, Any]]":
        """Returns a list of dictionaries representing the rows of the query result."""
        raise PySQLxError()
    async def query_first_as_dict(
        self, sql: str, params: Union[tuple, None]
    ) -> "Dict[str, Any]":
        """Returns the first row of the query result as a dictionary."""
        raise PySQLxError()
    async def raw_cmd(self, sql: str) -> "None":
        """Run a command in the database, for queries that can't be run using prepared statements."""
        raise PySQLxError()
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
        raise PySQLxError()
    async def start_transaction(
        self, isolation_level: "Union[IsolationLevel, None]"
    ) -> "None":
        """Starts a transaction with BEGIN. by default, does not set the isolation level."""
        raise PySQLxError()

async def new(uri: str) -> "Connection":
    """Creates a new connection to the database. Returns a `Connection` object."""
    raise PySQLxError()
