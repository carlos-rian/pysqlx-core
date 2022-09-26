use std::fmt::Debug;
use thiserror::Error;

#[derive(Error)]
pub enum DBError {
    #[error("RawQuery(code={0}, message='{1}')")]
    RawQuery(String, String),
    #[error("ConnectionError(code={0}, message='{1}')")]
    ConnectionError(String, String),
    #[error("ConversionError(message='could not convert from `{0}` to `{1}`')")]
    ConversionError(&'static str, &'static str),
}

impl Debug for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::RawQuery(code, msg) => write!(f, "RawQuery(code={}, message='{}')", code, msg),
            DBError::ConnectionError(code, msg) => {
                write!(f, "ConnectionError(code={}, message='{}')", code, msg)
            }
            DBError::ConversionError(from, to) => {
                write!(
                    f,
                    "ConversionError(message='could not convert from `{0}` to `{1}`')",
                    from, to
                )
            }
        }
    }
}
