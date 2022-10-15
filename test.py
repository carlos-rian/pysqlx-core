import asyncio
from time import time as now
import pysqlx_core


from datetime import date, datetime, time
from decimal import Decimal
from typing import Any, Dict, List
from uuid import UUID

from pydantic import Json, create_model, parse_obj_as


TYPES = {
    "bool": bool,
    "bytes": str,  # base64
    "null": Any,
    #
    "enum": str,  # Enum
    "str": str,
    #
    "int": int,
    "float": float,
    #
    "time": time,
    "date": date,
    "datetime": datetime,
    #
    "uuid": UUID,
    #
    "json": Json,
    "list": list,
    "xml": str,
    # not implement
}


async def main():
    conn = await pysqlx_core.new(uri="postgresql://postgres:postgrespw@localhost:49153")

    #check is_healthy
    print(conn.is_healthy())

    sql = "SELECT * FROM test;"

    # test query with list
    rows = await conn.query_py_obj(sql)
    print(rows)

    # test query with PysqlxRow
    rows = await conn.query(sql)
    all = rows.get_all()
    first = rows.get_first()
    types = rows.get_types()
    print(all, first, types, sep="\n\n\n")

    # test serializer

    for key, value in types.copy().items():
        types.update({key: (TYPES.get(value, Any), None)})
    model = create_model("Model", **types)
    new = parse_obj_as(List[model], all)
    print(new)

ini = now()
asyncio.run(main())
print(now() - ini)
