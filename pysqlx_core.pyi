from typing import Any, Dict, List
__all__ = ('PysqlxDBError', "Connection", "connect")

class Connection:
    async def query(self, sql: str, *args: str) -> None:
        ...

    async def execute(self, sql: str) -> "int":
        ...
    
class PysqlxDBError(Exception):
    code: str
    error: str

async def new(uri: str) -> 'Connection':
    raise PysqlxDBError()
