import asyncio
from unittest import result
import pysqlx_core

#await pysqlx_core.connect(uri="postgresql://postgres:password@localhost:5432/fastapi_pr")
exc = None

async def main():
    try:
        conn = await pysqlx_core.connect(uri="postgresql://postgres:password@localhost:5432/fastapi_prisma")
        sql = "SELECT * FROM peoples"
        result = await pysqlx_core.query(conn=conn, sql=sql)
        print(result.)
    except Exception as e:
        exc = e
        print(e)
    
    print(conn)
        
asyncio.run(main())
