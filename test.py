import asyncio
from time import time
import pysqlx_core

async def main():
    conn = await pysqlx_core.new(uri="postgresql://postgres:password@localhost:5432/fastapi_prisma?schema=public")
    
    rows = await conn.query("SELECT * FROM peoples")
    #types = conn.get_types()
    print(rows)
    #print(types)

    #await conn.query_lazy("SELECT * FROM peoples")
    #rows = conn.get_result()
    #types = conn.get_types()
    #print(rows)
    #print(types)

    #rows = await conn.query_one("SELECT * FROM peoples")
    #types = conn.get_types()
    #print(rows)
    #print(types)

ini = time()
asyncio.run(main())
print(time() - ini)
