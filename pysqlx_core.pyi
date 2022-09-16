
__all__ = ('Test',)

class Test:
    name: str
    age: int
    status: bool
    def __init__(self, name: str, age: int, status: bool) -> None: ...
    def __str__(cls) -> str: ...
