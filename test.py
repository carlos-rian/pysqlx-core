import asyncio
from time import time
import pysqlx_core

async def main():
    conn = await pysqlx_core.new(uri="postgresql://postgres:password@localhost:5432/fastapi_prisma?schema=public")
    rows = await conn.query("SELECT * FROM peoples")
    #print(rows)
ini = time()
asyncio.run(main())
print(time() - ini)
