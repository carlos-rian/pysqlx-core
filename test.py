import asyncio
from time import time
import pysqlx_core

#await pysqlx_core.connect(uri="postgresql://postgres:password@localhost:5432/fastapi_pr")
exc = None

async def main():
    conn = await pysqlx_core.connect(uri="postgresql://postgres:password@localhost:5432/fastapi_prisma?schema=public")
    result = await pysqlx_core.query(conn, "SELECT * FROM peoples")
    #print(result.get_types())
ini = time()
asyncio.run(main())
print(time() - ini)
