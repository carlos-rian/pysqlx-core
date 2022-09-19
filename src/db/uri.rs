
// Mssql Microsoft SQL Server
// Mysql
// Postgresql
// Sqlite
#[derive(Debug)]
pub enum Provider {
    Mssql,
    Mysql,
    Postgresql,
    Sqlite,
}

#[derive(Debug)]
pub struct Uri {
    pub uri: String,
    pub provider: Provider
}

impl Uri {
    pub fn new(uri: String, provider: Provider) -> Self { Self { uri, provider } }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    pub fn uri(&self) -> &str {
        self.uri.as_ref()
    }
}


#[derive(Debug, PartialEq)]
pub enum PySQLXProviderError {
    Invalid,
    Unknown
}