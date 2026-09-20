#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    DivideByZero {
        site: Site,
    },
    DivideOverflow {
        site: Site,
    },
    IndexOutOfRange {
        site: Site,
        index: i64,
        limit: i64,
    },
    KeyNotFound {
        site: Site,
        key: i64,
    },
    HostMissing {
        site: Site,
    },
    InvalidArgument {
        site: Site,
    },
    NullPointer {
        site: Site,
    },
    BadFunctionCall {
        site: Site,
    },
    OutOfRange {
        site: Site,
    },
    Unrepresentable {
        site: Site,
        reason: &'static str,
    },
}

impl Fault {
    #[track_caller]
    pub fn divide(divisor: i64) -> Self {
        if divisor == 0 {
            Self::DivideByZero { site: here() }
        } else {
            Self::DivideOverflow { site: here() }
        }
    }

    #[track_caller]
    pub fn divide_by_zero() -> Self {
        Self::DivideByZero { site: here() }
    }

    #[track_caller]
    pub fn divide_overflow() -> Self {
        Self::DivideOverflow { site: here() }
    }

    #[track_caller]
    pub fn index_out_of_range(index: i64, limit: i64) -> Self {
        Self::IndexOutOfRange {
            site: here(),
            index,
            limit,
        }
    }

    #[track_caller]
    pub fn key_not_found(key: i64) -> Self {
        Self::KeyNotFound {
            site: here(),
            key,
        }
    }

    #[track_caller]
    pub fn host_missing() -> Self {
        Self::HostMissing { site: here() }
    }

    #[track_caller]
    pub fn invalid_argument() -> Self {
        Self::InvalidArgument { site: here() }
    }

    #[track_caller]
    pub fn null_pointer() -> Self {
        Self::NullPointer { site: here() }
    }

    #[track_caller]
    pub fn bad_function_call() -> Self {
        Self::BadFunctionCall { site: here() }
    }

    #[track_caller]
    pub fn out_of_range() -> Self {
        Self::OutOfRange { site: here() }
    }

    #[track_caller]
    pub fn unrepresentable(reason: &'static str) -> Self {
        Self::Unrepresentable {
            site: here(),
            reason,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Site {
    file: &'static str,
    line: u32,
}

impl std::fmt::Display for Site {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}

#[track_caller]
fn here() -> Site {
    let caller = std::panic::Location::caller();

    Site {
        file: site_of(caller.file()),
        line: caller.line(),
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivideByZero { site } => write!(f, "{site} divided by zero"),
            Self::DivideOverflow { site } => {
                write!(f, "{site} produced a quotient too large to store")
            }
            Self::KeyNotFound { site, key } => {
                write!(f, "{site} looked up {key}, which its table does not hold")
            }
            Self::Unrepresentable { site, reason } => {
                write!(
                    f,
                    "{site} stopped where the game itself would have carried on: {reason}"
                )
            }
            Self::HostMissing { site } => write!(f, "{site} has no host attached"),
            Self::NullPointer { site } => write!(f, "{site} followed a pointer that was never set"),
            Self::BadFunctionCall { site } => {
                write!(f, "{site} called a handler that was never set")
            }
            Self::InvalidArgument { site } => {
                write!(f, "{site} was given text that holds no number")
            }
            Self::OutOfRange { site } => write!(f, "{site} was given a number too large to store"),
            Self::IndexOutOfRange { site, index, limit } => {
                write!(f, "{site} reached {index}, past its limit of {limit}")
            }
        }
    }
}

impl std::error::Error for Fault {}

const fn site_of(path: &'static str) -> &'static str {
    let bytes = path.as_bytes();
    let mut start = 0;
    let mut at = 0;

    while at < bytes.len() {
        if bytes[at] == b'/' || bytes[at] == b'\\' {
            start = at + 1;
        }

        at += 1;
    }

    let (_, name) = bytes.split_at(start);

    if name.len() < 4 {
        return path;
    }

    let (stem, _) = name.split_at(name.len() - 3);

    match str::from_utf8(stem) {
        Ok(text) => text,
        Err(_) => path,
    }
}
