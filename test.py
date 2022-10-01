import asyncio
from lib2to3.pytree import Base
import pysqlx_core

#await pysqlx_core.connect(uri="postgresql://postgres:password@localhost:5432/fastapi_pr")
exc = None

async def main():
    try:
        conn = await pysqlx_core.connect(uri="postgresql://postgres:password@localhost:5432/fastapi_pr")
    except Exception as e:
        exc = e
        print(e)
        
asyncio.run(main())
