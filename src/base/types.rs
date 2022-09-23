
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum TypeIdentifier {
    String,
    Int,
    BigInt,
    Float,
    Decimal,
    Boolean,
    Enum(String),
    UUID,
    Json,
    Xml,
    DateTime,
    Bytes,
    Unsupported,
}

impl TypeIdentifier {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            TypeIdentifier::Int | TypeIdentifier::BigInt | TypeIdentifier::Float | TypeIdentifier::Decimal
        )
    }
}

impl std::fmt::Display for TypeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeIdentifier::String => write!(f, "String"),
            TypeIdentifier::Int => write!(f, "Int"),
            TypeIdentifier::BigInt => write!(f, "BigInt"),
            TypeIdentifier::Float => write!(f, "Float"),
            TypeIdentifier::Decimal => write!(f, "Decimal"),
            TypeIdentifier::Boolean => write!(f, "Bool"),
            TypeIdentifier::Enum(e) => write!(f, "Enum{}", e),
            TypeIdentifier::UUID => write!(f, "UUID"),
            TypeIdentifier::Json => write!(f, "Json"),
            TypeIdentifier::Xml => write!(f, "Xml"),
            TypeIdentifier::DateTime => write!(f, "DateTime"),
            TypeIdentifier::Bytes => write!(f, "Bytes"),
            TypeIdentifier::Unsupported => write!(f, "Unsupported"),
        }
    }
}