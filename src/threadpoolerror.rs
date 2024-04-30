pub type Result<T> = std::result::Result<T, ThreadPoolError>;

#[derive(Debug, Clone)]
pub enum ThreadPoolErrorReason {
    Other(String),
    InvalidPoolSize,
    InvalidDynamicPoolBounds,
    DynamicResizingError,
}

impl std::fmt::Display for ThreadPoolErrorReason {
    fn fmt(&self, f : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadPoolErrorReason::Other(s) => write!(f, "Other: {}", s),
            ThreadPoolErrorReason::InvalidPoolSize => write!(f, "InvalidPoolSize"),
            ThreadPoolErrorReason::InvalidDynamicPoolBounds => write!(f, "InvalidDynamicPoolBounds"),
            ThreadPoolErrorReason::DynamicResizingError => write!(f, "DynamicResizingError")
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThreadPoolError {
    reason: ThreadPoolErrorReason,
}

impl ThreadPoolError {
    pub fn new(reason: ThreadPoolErrorReason) -> ThreadPoolError {
        ThreadPoolError { reason }
    }
}

impl std::fmt::Display for ThreadPoolError {
    fn fmt(&self, f : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ThreadPoolError: {}", self.reason)
    }
}
