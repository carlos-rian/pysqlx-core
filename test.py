import asyncio
from time import time as now
import pysqlx_core


from datetime import date, datetime, time
from decimal import Decimal
from typing import Any, Dict, List
from uuid import UUID

from pydantic import Json, create_model, parse_obj_as


TYPES = {
    "int": int,
    "bigint": int,
    "float": float,
    "double": float,
    "string": str,
    "bool": bool,
    "char": str,
    "decimal": Decimal,
    "json": Json,
    "uuid": UUID,
    "datetime": datetime,
    "date": date,
    "time": time,
    "array": list,
    "xml": str,
    # not implement
    "bytes": Any,  # bytes
    "enum": Any,  # Enum
    "null": Any,
}


async def main():
    conn = await pysqlx_core.new(uri="postgresql://postgres:password@localhost:5432/fastapi_prisma?schema=public")
    
    rows = await conn.query("SELECT * FROM peoples")
    all = rows.get_all()
    first = rows.get_first()
    types = rows.get_types()
    for key, value in types.copy().items():
        types.update({key: (TYPES.get(value, Any), None)})
    model = create_model("Model", **types)
    new = parse_obj_as(List[model], all)
    print(new)

ini = now()
asyncio.run(main())
print(now() - ini)
