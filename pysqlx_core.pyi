from typing import Any, Dict, List
__all__ = ('Test',)

class PysqlRows:
    def get_type(self) -> 'Dict[str, Any]': ...

class PyConnection:
    ...

async def connect(uri: str) -> 'PyConnection':
    ...

async def query(conn: PyConnection, sql: str) -> 'List[Dict[str, Any]]':
    ...
    
class PysqlxDBError(Exception):
    code: str
    error: str
