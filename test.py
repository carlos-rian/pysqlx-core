from pysqlx_core import Test

c = Test(name="carlos", age=28, status=True)
print(c)

class X:
    def __init__(self, l: list) -> None:
        self.l: list = l

    def __add__(self, l2):
        return self.l + l2
