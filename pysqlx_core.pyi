from typing import Dict
__all__ = ('PysqlRows', "PyConnection", "connect", "query")

class PysqlRows:
    def get_types(self) -> 'Dict[str, str]': ...

class PyConnection:
    ...
    
class PysqlxDBError(Exception):
    code: str
    error: str

async def connect(uri: str) -> 'PyConnection':
    ...

async def query(conn: PyConnection, sql: str) -> 'PysqlRows':
    ...
    
async def test_query() -> 'list[dict[str, str]]':
    ...
    