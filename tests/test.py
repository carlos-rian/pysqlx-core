from datetime import datetime, date, time
from uuid import UUID
from enum import Enum
from pysqlx_core import new, PySQLxStatement
import asyncio
import uvloop
from pprint import pprint
import logging
from decimal import Decimal

Decimal()

logging.basicConfig(level=logging.DEBUG)


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
    await conn.execute(PySQLxStatement(provider="sqlite", sql="DROP TABLE IF EXISTS users"))
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

    result = await conn.query_typed(PySQLxStatement(provider="sqlite", sql="SELECT * FROM users"))
    pprint(result.get_all())
    pprint(result.get_first())
    pprint(result.get_last_insert_id())
    pprint(len(result))


def typ():
    class EnumColors(Enum):
        BLUE = "blue"
        RED = "red"
        GRAY = "gray"
        BLACK = "black"

    return {
        "type_int": 1957483605,  # int
        "type_smallint": 3618,  # int (smallint)
        "type_bigint": -3762183126230668,  # int (bigint)
        "type_serial": 36,  # int (serial)
        "type_smallserial": 36,  # int (smallserial)
        "type_bigserial": 36,  # int (bigserial)
        "type_numeric": 130.3064,  # float (numeric)
        "type_float": 2159.912,  # float (float)
        "type_double": 1577.3155,  # float (double)
        "type_money": 6803.77,  # float (money), parsed from string '$6,803.77'
        "type_char": "C",  # str (char)
        "type_varchar": "ATYOLOUREPOJRSNOWKMULTTRHJPTCWOIYHQVVIXVUFZNCMEFJTRLCJZMKNJVAUYIEZYKVPWCWGGRDBUKKEDQHSEYPACMNGBOLHLC",  # str (varchar)
        "type_text": "text",  # str (text)
        "type_boolean": False,  # bool (boolean)
        "type_date": date.fromisoformat("2022-10-27"),  # str (date)
        "type_time": time.fromisoformat("00:00:21"),  # str (time)
        "type_datetime": datetime.fromisoformat("2022-10-27 15:29:27.000000"),
        "type_datetimetz": datetime.utcnow(),
        "type_enum": EnumColors.BLUE,
        "type_uuid": UUID("19b3d203-e4b7-4b7b-8bf2-476abea92b04"),
        "type_json": {"cep": "01001-000"},
        "type_jsonb": b'{"cep": "01001-000"}',
        "type_xml": "<note><to>Tove</to></note>",
        "type_inet": "192.168.0.1",
        "type_bytes": b"DEADBEEF",
        "type_array_text": ("name", "age"),
        "type_array_integer": (1, 2, 3),
        "type_array_date": (date(2022, 10, 27), date(2022, 10, 27)),
        "type_array_uuid": (UUID("7b97c8a6-7e5a-4412-a57d-78565a136582"), UUID("7b97c8a6-7e5a-4412-a57d-78565a136583")),
    }


async def psql():

    conn = await new("postgresql://postgres:Build!Test321@localhost:4442/engine")

    await conn.execute(PySQLxStatement(provider="postgresql", sql="DROP TABLE IF EXISTS pysqlx_table"))

    # create enum type
    await conn.execute(PySQLxStatement(provider="postgresql", sql="DROP TYPE IF EXISTS colors;"))
    await conn.execute(
        PySQLxStatement(provider="postgresql", sql="CREATE TYPE colors AS ENUM ('blue', 'red', 'gray', 'black');")
    )

    await conn.execute(
        PySQLxStatement(
            provider="postgresql",
            sql="""
                create table pysqlx_table
                (
                    type_int           integer,
                    type_smallint      smallint,
                    type_bigint        bigint,
                    type_serial        serial,
                    type_smallserial   smallserial,
                    type_bigserial     bigserial,
                    type_numeric       numeric,
                    type_float         double precision,
                    type_double        double precision,
                    type_money         money,
                    type_char          char,
                    type_varchar       varchar(100),
                    type_text          text,
                    type_boolean       boolean,
                    type_date          date,
                    type_time          time,
                    type_datetime      timestamp,
                    type_datetimetz    timestamp with time zone,
                    type_enum          colors,
                    type_uuid          uuid,
                    type_json          json,
                    type_jsonb         jsonb,
                    type_xml           xml,
                    type_inet          inet,
                    type_bytes         bytea,
                    type_array_text    text[],
                    type_array_integer integer[],
                    type_array_date    date[],
                    type_array_uuid    uuid[]
                );
            """,
        )
    )

    p = PySQLxStatement(
        provider="postgresql",
        sql="""
                INSERT INTO pysqlx_table (
                    type_int,
                    type_smallint,
                    type_bigint,
                    type_serial,
                    type_smallserial,
                    type_bigserial,
                    type_numeric,
                    type_float,
                    type_double,
                    type_money,
                    type_char,
                    type_varchar,
                    type_text,
                    type_boolean,
                    type_date,
                    type_time,
                    type_datetime,
                    type_datetimetz,
                    type_enum,
                    type_uuid,
                    type_json,
                    type_jsonb,
                    type_xml,
                    type_inet,
                    type_bytes,
                    type_array_text,
                    type_array_integer,
                    type_array_date,
                    type_array_uuid    
                )
                VALUES (
                    :type_int,
                    :type_smallint,
                    :type_bigint,
                    :type_serial,
                    :type_smallserial,
                    :type_bigserial,
                    :type_numeric,
                    :type_float,
                    :type_double,
                    :type_money,
                    :type_char,
                    :type_varchar,
                    :type_text,
                    :type_boolean,
                    :type_date,
                    :type_time,
                    :type_datetime,
                    :type_datetimetz,
                    :type_enum,
                    :type_uuid,
                    :type_json,
                    :type_jsonb,
                    :type_xml,
                    :type_inet,
                    :type_bytes,
                    :type_array_text,
                    :type_array_integer,
                    :type_array_date,
                    :type_array_uuid
                );
                """,
        params=typ(),
    )

    row_affected = await conn.execute(p)
    assert row_affected == 1

    result = await conn.query_typed(PySQLxStatement(provider="postgresql", sql="SELECT * FROM users"))
    pprint(result.get_all())
    pprint(result.get_first())
    pprint(result.get_last_insert_id())
    pprint(len(result))


async def main():
    await psql()
    await sqlite()


if __name__ == "__main__":
    # asyncio.run(main())

    uvloop.install()
    asyncio.run(main())

    # trio.run(main)
