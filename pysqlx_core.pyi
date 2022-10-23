from typing import Any, Dict, List
__all__ = ("__version__", "new", "Connection", 'PySQLXError', "PySQLXResult")
__version__: str


class PySQLXError(Exception):
    """
    ## PySQLXError

    ### Base class for all exceptions raised by pysqlx.
     
    `code`: str - mapped to the error code
    `error`: str - description of the error
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
        * RawQuery
        * ConnectionError
        * ConversionError

        """
        ...

class PySQLXResult:
    """
    PySQLXResult is a class that represents the result of a query.
    It is returned by the `query` method of the `Connection` class.
    """
    def get_model(self) -> "Dict[str, str]":
        """Returns a dictionary of column names and their types used to generate Pydantic BaseModel."""
        raise PySQLXError()
    
    def get_all(self) -> "List[Dict[str, Any]]":
        """Returns a list of dictionaries representing the rows of the query result."""
        raise PySQLXError()
    
    def get_first(self) -> "Dict[str, Any]":
        """Returns the first row of the query result as a dictionary."""
        raise PySQLXError()

class Connection:
    """Creates a new connection to the database. Create after calling `new`."""
    async def query(self, sql: str) -> "PySQLXResult":
        """Returns a `PySQLXResult` object representing the result of the query."""
        raise PySQLXError()
    
    async def execute(self, sql: str) -> "int":
        """Executes a query and returns the number of rows affected."""
        raise PySQLXError()

    async def query_as_list(self, sql: str) -> "List[Dict[str, Any]]":
        """Returns a list of dictionaries representing the rows of the query result."""
        raise PySQLXError()
    
    async def raw_cmd(self, sql: str) -> "str": 
        """Run a command in the database, for queries that can't be run using prepared statements."""
        ...

    def is_healthy(self) -> "bool": 
        """Returns false, if connection is considered to not be in a working state"""
        ...

    def requires_isolation_first(self) -> "bool": 
        """Returns `True` if the connection requires isolation first, `False` otherwise. 
        This is used to determine if the connection should be isolated before executing a query.
        for example, sqlserver requires isolation before executing a query using begin.
        
        - Signals if the isolation level SET needs to happen before or after the tx BEGIN

        * [SQL Server documentation]: (https://docs.microsoft.com/en-us/sql/t-sql/statements/set-transaction-isolation-level-transact-sql?view=sql-server-ver15)
        * [Postgres documentation]: (https://www.postgresql.org/docs/current/sql-set-transaction.html)
        * [MySQL documentation]: (https://dev.mysql.com/doc/refman/8.0/en/innodb-transaction-isolation-levels.html)
        * [SQLite documentation]: (https://www.sqlite.org/isolation.html)
        """
        ...

async def new(uri: str) -> 'Connection':
    """Creates a new connection to the database. Returns a `Connection` object."""
    raise PySQLXError()
