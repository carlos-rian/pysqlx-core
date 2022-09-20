use std::fmt;
use std::error;

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
pub struct Params {
    pub provider: Provider,
    pub uri: String
}

impl Params {
    pub fn new(provider: &str, uri: &str) -> Result<Self, PySqlxUriError> { 
        let provider_ = match provider {
            "sqlite" => Provider::Sqlite,
            "mysql" => Provider::Mysql,
            "mssql" => Provider::Mssql,
            "postgresql" => Provider::Postgresql,
            _ => return Err(PySqlxUriError::InvalidProvider)
        };
        if uri.starts_with(provider) {
            Ok(Self{uri: uri.to_string(), provider: provider_})
        } else {
            Err(PySqlxUriError::InvalidURI)
        }
    }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }

    pub fn uri(&self) -> &str {
        self.uri.as_ref()
    }
}


#[derive(Debug, PartialEq)]
pub enum PySqlxUriError {
    InvalidURI,
    InvalidProvider,
    Unknown
}

impl fmt::Display for PySqlxUriError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match *self {
            PySqlxUriError::InvalidProvider => "invalid uri, check and try again.",
            PySqlxUriError::InvalidURI => "invalid uri, check and try again.",
            PySqlxUriError::Unknown => "unknow error"
        };
        f.write_str(description)
    }
}

impl error::Error for PySqlxUriError {}