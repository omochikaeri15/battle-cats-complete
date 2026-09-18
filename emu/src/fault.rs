#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    DivideByZero { site: &'static str },
    DivideOverflow { site: &'static str },
    IndexOutOfRange { site: &'static str, index: i64, limit: i64 },
    KeyNotFound { site: &'static str, key: i64 },
    HostMissing { site: &'static str },
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivideByZero { site } => write!(f, "{site} divided by zero"),
            Self::DivideOverflow { site } => write!(f, "{site} produced a quotient too large to store"),
            Self::KeyNotFound { site, key } => write!(f, "{site} looked up {key}, which its table does not hold"),
            Self::HostMissing { site } => write!(f, "{site} has no host attached"),
            Self::IndexOutOfRange { site, index, limit } => {
                write!(f, "{site} reached {index}, past its limit of {limit}")
            }
        }
    }
}

impl std::error::Error for Fault {}
