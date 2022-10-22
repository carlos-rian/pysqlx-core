# pysqlx-core


This package provides the core functionality for PySQLX-Engine.

The package is currently a work in progress and subject to significant change.

__pysqlx-core__will be a separate package, required by `pysqlx-engine`.

This core is not so friendly, but maybe you want to use it, feel free to suggest improvements.


Example of usage:

```python
import pysqlx_core
import asyncio

async def main(sql):
    db = await pysqlx_core.new(uri="postgresql://postgres:postgrespw@localhost:49153")
    

    # Create a table
    await db.execute(sql="""
        CREATE TABLE IF NOT EXISTS test (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL
        );
        """
    )

    # Insert a row
    await db.execute(sql="INSERT INTO test (name) VALUES ('Carlos');")

    # Select all rows, return a class PySQLXResult
    result = await db.query(sql="SELECT * FROM test;")
    # get first row
    row = result.get_first() # Dict[str, Any] 
    # get all rows
    rows = result.get_all() # List[Dict[str, Any]]
    #return the db types to Pydantic BaseModel
    types = result.get_model() # Dict[str, str] 

    # Select all rows, return with List[Dict[str, Any]]
    rows = await db.query_as_list(sql="SELECT * FROM test;")

    # close? no need 👌-> auto-close when finished programmer or go out of context..
    
asyncio.run(main())
```

