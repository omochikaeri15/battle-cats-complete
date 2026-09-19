#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    DivideByZero { site: &'static str },
    DivideOverflow { site: &'static str },
    IndexOutOfRange { site: &'static str, index: i64, limit: i64 },
    KeyNotFound { site: &'static str, key: i64 },
    HostMissing { site: &'static str },
    InvalidArgument { site: &'static str },
    NullPointer { site: &'static str },
    BadFunctionCall { site: &'static str },
    OutOfRange { site: &'static str },
    Unrepresentable { site: &'static str, reason: &'static str },
}

impl Fault {
    pub fn divide(site: &'static str, divisor: i64) -> Self {
        if divisor == 0 {
            Self::DivideByZero { site }
        } else {
            Self::DivideOverflow { site }
        }
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivideByZero { site } => write!(f, "{site} divided by zero"),
            Self::DivideOverflow { site } => write!(f, "{site} produced a quotient too large to store"),
            Self::KeyNotFound { site, key } => write!(f, "{site} looked up {key}, which its table does not hold"),
            Self::Unrepresentable { site, reason } => {
                write!(f, "{site} stopped where the game itself would have carried on: {reason}")
            }
            Self::HostMissing { site } => write!(f, "{site} has no host attached"),
            Self::NullPointer { site } => write!(f, "{site} followed a pointer that was never set"),
            Self::BadFunctionCall { site } => write!(f, "{site} called a handler that was never set"),
            Self::InvalidArgument { site } => write!(f, "{site} was given text that holds no number"),
            Self::OutOfRange { site } => write!(f, "{site} was given a number too large to store"),
            Self::IndexOutOfRange { site, index, limit } => {
                write!(f, "{site} reached {index}, past its limit of {limit}")
            }
        }
    }
}

impl std::error::Error for Fault {}
