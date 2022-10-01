
__all__ = ('Test',)

class Connection:
    ...

async def connect(uri: str) -> 'Connection':
    ...
    
class PysqlxDBError(Exception):
    code: str
    error: str
