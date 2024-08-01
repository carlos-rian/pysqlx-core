from datetime import datetime
from enum import Enum
from pysqlx_core import new, PySQLxStatement
import asyncio
from pprint import pprint


async def sqlite():
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

    result = await conn.query_typed(
        PySQLxStatement(provider="sqlite", sql="SELECT * FROM users")
    )
    pprint(result.get_all())
    pprint(result.get_first())
    pprint(result.get_last_insert_id())
    pprint(len(result))


async def psql():
    conn = await new("postgresql://postgres:Build!Test321@localhost:4442/engine")

    class EnumExample(Enum):
        A = "A"
        B = "B"
        C = "C"

    await conn.execute(
        PySQLxStatement(provider="postgresql", sql="DROP TABLE IF EXISTS users")
    )

    # create enum type
    await conn.execute(
        PySQLxStatement(provider="postgresql", sql="DROP TYPE IF EXISTS EXAMPLE;")
    )
    await conn.execute(
        PySQLxStatement(
            provider="postgresql",
            sql="CREATE TYPE EXAMPLE AS ENUM ('A', 'B', 'C')",
        )
    )

    await conn.execute(
        PySQLxStatement(
            provider="postgresql",
            sql="""
                CREATE TABLE IF NOT EXISTS users (
                    id SERIAL PRIMARY KEY,
                    name TEXT NOT NULL,
                    created_at TIMESTAMP,
                    bool BOOLEAN,
                    int INTEGER,
                    float REAL,
                    enum_example EXAMPLE
                )
            """,
        )
    )
    params = [
        ("John Do", datetime.now(), True, 1, 1.1, EnumExample.A),
        ("Jane Do", datetime.now(), False, 2, 2.2, EnumExample.B),
        ("Alice F", datetime.now(), True, 3, 3.3, EnumExample.C),
        ("Bob Dow", datetime.now(), False, 4, 4.4, EnumExample.C),
        ("Charlie", datetime.now(), True, 5, 5.5, EnumExample.B),
    ]

    for param in params:
        row_affected = await conn.execute(
            PySQLxStatement(
                provider="postgresql",
                sql="INSERT INTO users (name, created_at, bool, int, float, enum_example) VALUES (:name, :created_at, :bool, :int, :float, :enum_example)",
                params=dict(
                    zip(
                        ["name", "created_at", "bool", "int", "float", "enum_example"],
                        param,
                    )
                ),
            )
        )
        assert row_affected == 1

    result = await conn.query_typed(
        PySQLxStatement(provider="postgresql", sql="SELECT * FROM users")
    )
    pprint(result.get_all())
    pprint(result.get_first())
    pprint(result.get_last_insert_id())
    pprint(len(result))


async def main():
    await psql()
    await sqlite()


if __name__ == "__main__":
    asyncio.run(main())
