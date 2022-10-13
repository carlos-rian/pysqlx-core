from typing import Any, Dict, List
__all__ = ('PysqlxDBError', "Connection", "connect")

class Connection:
    async def query(self, sql: str) -> "List[Dict[str, Any]]":
        raise PysqlxDBError()
    
    async def query_one(self, sql: str) -> "Dict[str, Any]" | None:
        raise PysqlxDBError()

    async def query_lazy(self, sql: str) -> None:
        raise PysqlxDBError()

    async def execute(self, sql: str) -> "int":
        raise PysqlxDBError()
    
    def get_result(self) -> "List[Dict[str, Any]]" | None:
        raise PysqlxDBError()
    
    def get_types(self) -> "List[Dict[str, str]]" | None:
        raise PysqlxDBError()
    
class PysqlxDBError(Exception):
    code: str
    error: str

async def new(uri: str) -> 'Connection':
    raise PysqlxDBError()
