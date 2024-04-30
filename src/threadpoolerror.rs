pub type Result<T> = std::result::Result<T, ThreadPoolError>;

#[derive(Debug, Clone)]
pub enum ThreadPoolErrorReason {
    Other(String),
    InvalidPoolSize,
    InvalidDynamicPoolBounds,
    DynamicResizingError,
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
