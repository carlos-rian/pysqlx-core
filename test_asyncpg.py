import asyncio
from time import time
import asyncpg

async def main():
    conn = await asyncpg.connect(user='postgres', password='password',
                                 database='fastapi_prisma', host='127.0.0.1')
    values = await conn.fetch(
        'SELECT * FROM table_test'
    )
    print(values)
    await conn.close()

ini = time()
asyncio.run(main())
print(time() - ini)
