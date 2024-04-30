pub type Result<T> = std::result::Result<T, ThreadPoolError>;

#[derive(Debug, Clone)]
enum ThreadPoolErrorReason {
    Other(String),
    InvalidPoolSize,
    InvalidDynamicPoolBounds,
}

#[derive(Debug, Clone)]
pub struct ThreadPoolError {
    reason: ThreadPoolErrorReason,
}
