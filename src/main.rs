use pysqlx_core::db::uri::Uri;
use pysqlx_core::db::uri::Provider;
use pysqlx_core::db::uri::PySQLXProviderError;

fn connect(prov: &str, uri: &str) -> Result<Uri, PySQLXProviderError> {
    let provider = match prov {
        "sqlite" => Provider::Sqlite,
        "mysql" => Provider::Mysql,
        "mssql" => Provider::Mssql,
        "postgresql" => Provider::Postgresql,
        _ => return Err(PySQLXProviderError::Invalid)

    };

    Ok(Uri{uri: uri.to_string(), provider: provider})
}

fn main(){
    let uri = connect("sqlite", "sqlite::memory:");
    print!("{:?}", uri);
}