import asyncio
from time import time
import asyncpg

async def main():
    conn = await asyncpg.connect(user='postgres', password='postgrespw',
                                 database='postgres', host='127.0.0.1', port="49153")
    values = await conn.fetch(
        'SELECT * FROM test'
    )
    print(values)
    await conn.close()

ini = time()
asyncio.run(main())
print(time() - ini)
