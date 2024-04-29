use std::fmt;
use rustdns::Rcode;

pub type Result<T> = std::result::Result<T, DnsError>;

#[derive(Debug, Clone)]
pub struct DnsError {
    code : Rcode
}

impl fmt::Display for DnsError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DNS Error. Code {}", self.code)
    }
}

impl DnsError {
    pub fn new(code : Rcode) -> Self {
        Self { code}
    }
}
