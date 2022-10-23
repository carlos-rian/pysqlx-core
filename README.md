# pysqlx-core

pysqlx-core is an extremely fast Python library for communicating with various SQL databases.

This package provides the core functionality for [PySQLX-Engine](https://carlos-rian.github.io/pysqlx-engine/).

The package is currently a work in progress and subject to significant change.

[__pysqlx-core__](https://pypi.org/project/pysqlx-core/) will be a separate package, required by [pysqlx-engine](https://carlos-rian.github.io/pysqlx-engine/)`.

This package is written entirely in Rust and compiled as a Python library using PyO3 and PyO3-Asyncio.

This core is not so friendly, but maybe you want to use it, feel free to suggest improvements.

### Supported databases

* [`SQLite`](https://www.sqlite.org/index.html)
* [`PostgreSQL`](https://www.postgresql.org/)
* [`MySQL`](https://www.mysql.com/)
* [`Microsoft SQL Server`](https://www.microsoft.com/sql-server)

### Supported Python versions

* [`Python >= 3.7`](https://www.python.org/)

### Supported operating systems

* [`Linux`](https://pt.wikipedia.org/wiki/Linux)
* [`MacOS`](https://pt.wikipedia.org/wiki/Macos)
* [`Windows`](https://pt.wikipedia.org/wiki/Microsoft_Windows)


### Example of installation:

PIP

```bash
$ pip install pysqlx-engine
```

Poetry

```bash
$ poetry add pysqlx-engine
```

### Example of usage:

```python
import pysqlx_core
import asyncio

async def main(sql):
    # Create a connection 
    db = await pysqlx_core.new(uri="postgresql://postgres:postgrespw@localhost:49153")
    
    # Create a table
    await db.execute(sql="""
        CREATE TABLE IF NOT EXISTS test (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL
        );
        """
    )

    # Insert a row and return quantity rows affected
    await db.execute(sql="INSERT INTO test (name) VALUES ('Carlos');")

    # Select all rows, return a class PySQLXResult
    result = await db.query(sql="SELECT * FROM test;")
    # get first row
    row = result.get_first() # Dict[str, Any] 
    # get all rows
    rows = result.get_all() # List[Dict[str, Any]]
    #return the db types to Pydantic BaseModel
    types = result.get_model() # Dict[str, str] 

    # Select all rows, return how List[Dict[str, Any]]
    rows = await db.query_as_list(sql="SELECT * FROM test;")

    # close? no need 👌-> auto-close when finished programmer or go out of context..
    
asyncio.run(main())
```