from datetime import datetime
from pysqlx_core import new, PySQLxStatement
import asyncio
from pprint import pprint


async def main():
    conn = await new("file:///tmp/db.db")

    tb = PySQLxStatement(
        provider="sqlite",
        sql="""
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TIMESTAMP,
            bool BOOLEAN,
            int INTEGER,
            float REAL
        )
    """,
    )
    await conn.execute(
        PySQLxStatement(provider="sqlite", sql="DROP TABLE IF EXISTS users")
    )
    await conn.execute(tb)
    params = [
        ("John Do", datetime.now(), True, 1, 1.1),
        ("Jane Do", datetime.now(), False, 2, 2.2),
        ("Alice F", datetime.now(), True, 3, 3.3),
        ("Bob Dow", datetime.now(), False, 4, 4.4),
        ("Charlie", datetime.now(), True, 5, 5.5),
    ]

    for param in params:
        row_affected = await conn.execute(
            PySQLxStatement(
                provider="sqlite",
                sql="INSERT INTO users (name, created_at, bool, int, float) VALUES (:name, :created_at, :bool, :int, :float)",
                params=dict(zip(["name", "created_at", "bool", "int", "float"], param)),
            )
        )
        assert row_affected == 1

    result = await conn.query(
        PySQLxStatement(provider="sqlite", sql="SELECT * FROM users")
    )
    pprint(result.get_all())
    pprint(result.get_first())
    pprint(result.get_last_insert_id())
    pprint(len(result))


if __name__ == "__main__":
    asyncio.run(main())
