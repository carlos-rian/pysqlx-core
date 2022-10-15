from typing import Any, Dict, List
__all__ = ("new", "Connection", 'PysqlxDBError', "PysqlxRows")

class PysqlxDBError(Exception):
    code: str
    error: str

class PysqlxRows:
    def get_types(self) -> "Dict[str, str]":
        raise PysqlxDBError()
    
    def get_all(self) -> "List[Dict[str, Any]]":
        raise PysqlxDBError()
    
    def get_first(self) -> "Dict[str, Any]":
        raise PysqlxDBError()

class Connection:
    async def query(self, sql: str) -> "PysqlxRows":
        raise PysqlxDBError()
    
    async def execute(self, sql: str) -> "int":
        raise PysqlxDBError()

    async def query_py_obj(self, sql: str) -> "List[Dict[str, Any]]":
        raise PysqlxDBError()
    
    def is_healthy(self) -> "bool":
        raise PysqlxDBError()

async def new(uri: str) -> 'Connection':
    raise PysqlxDBError()
