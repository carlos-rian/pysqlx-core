use std::fmt::{self, Display};

use super::value::PySQLxValue;

pub(crate) struct Params<'a>(pub(crate) &'a [PySQLxValue]);

impl<'a> Display for Params<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.0.len();

        write!(f, "[")?;
        for (i, val) in self.0.iter().enumerate() {
            let v = val.clone().to_value();
            write!(f, "{v}")?;

            if i < (len - 1) {
                write!(f, ",")?;
            }
        }
        write!(f, "]")
    }
}
