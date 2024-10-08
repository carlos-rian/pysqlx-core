# __pysqlx-core__

[![cargo ci](https://github.com/carlos-rian/pysqlx-core/workflows/ci/badge.svg?branch=main)](https://github.com/carlos-rian/pysqlx-core/actions?query=event%3Apush+branch%3Amain+workflow%3Aci)
[![pypi](https://img.shields.io/pypi/v/pysqlx-core.svg?color=%2334D058)](https://pypi.python.org/pypi/pysqlx-core)
[![versions](https://img.shields.io/pypi/pyversions/pysqlx-core.svg?color=%2334D058)](https://github.com/carlos-rian/pysqlx-core)
[![license](https://img.shields.io/github/license/carlos-rian/pysqlx-core.svg?color=%2334D058)](https://github.com/carlos-rian/pysqlx-core/blob/main/LICENSE)
[![downloads](https://static.pepy.tech/personalized-badge/pysqlx-core?period=total&units=international_system&left_color=grey&right_color=brightgreen&left_text=downloads)](https://pepy.tech/project/pysqlx-core)

pysqlx-core is an extremely fast Python library for communicating with various SQL databases.

This package provides the core functionality for [__PySQLX-Engine__](https://carlos-rian.github.io/pysqlx-engine/).

The package is currently a work in progress and subject to significant change.

[__pysqlx-core__](https://pypi.org/project/pysqlx-core/) will be a separate package, required by [__pysqlx-engine__](https://carlos-rian.github.io/pysqlx-engine/).

This package is written entirely in Rust and compiled as a Python library using PyO3 and PyO3-Asyncio.

This core is not so friendly, but maybe you want to use it, feel free to suggest improvements.

### Supported databases

* [__`SQLite`__](https://www.sqlite.org/index.html)
* [__`PostgreSQL`__](https://www.postgresql.org/)
* [__`MySQL`__](https://www.mysql.com/)
* [__`Microsoft SQL Server`__](https://www.microsoft.com/sql-server)

### Supported Python versions

* [__`Python >= 3.8`__](https://www.python.org/)

### Supported operating systems

* [__`Linux`__](https://pt.wikipedia.org/wiki/Linux)
* [__`MacOS`__](https://pt.wikipedia.org/wiki/Macos)
* [__`Windows`__](https://pt.wikipedia.org/wiki/Microsoft_Windows)


### Example of installation:

__PIP__

```bash
$ pip install pysqlx-core
```

__Poetry__

```bash
$ poetry add pysqlx-core
```

### Example of usage:

```python
import pysqlx_core
import asyncio

async def main(sql):
    # Create a connection 
    db = await pysqlx_core.new(uri="postgresql://postgres:postgrespw@localhost:49153")
    
    # Create a table
    stmt = pysqlx_core.PySQLxStatement(
        provider="postgresql", 
        sql="""
            CREATE TABLE IF NOT EXISTS test (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            );
        """)
    await db.execute(stmt=stmt)

    # Insert a row and return quantity rows affected
    insert = pysqlx_core.PySQLxStatement(
        provider="postgresql", 
        sql="INSERT INTO test (name) VALUES (:name);",
        params={"name": "Carlos"}
    )
    await db.execute(stmt=insert)

    # can you see the sql and params pre builded
    print("SQL:", insert.sql())
    # output: INSERT INTO test (name) VALUES ($1);
    print("PARAMS:", insert.params())
    # output: ['Carlos']

    # Select all rows, return a class PySQLxResponse
    result = await db.query_typed(stmt=pysqlx_core.PySQLxStatement(
            provider="postgresql", 
            sql="SELECT * FROM test;"
        )
    )
    # get first row
    row = result.get_first() # Dict[str, Any] 
    # get all rows
    rows = result.get_all() # List[Dict[str, Any]]
    # return the db 'types' to Pydantic BaseModel
    types = result.get_types() # Dict[str, str] 

    # Select all rows, return how List[Dict[str, Any]]
    rows = await db.query_all(pysqlx_core.PySQLxStatement(provider="postgresql", sql="SELECT * FROM test;"))

    # close? no need 👌-> auto-close when finished programmer or go out of context..
    
asyncio.run(main())
```
