use std::fmt;
use rustdns::Rcode;

pub type Result<T> = std::result::Result<T, DnsError>;

#[derive(Debug, Clone)]
pub struct DnsError {
    code : Rcode,
    info : String
}

impl fmt::Display for DnsError {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DNS Error. Code {}", self.code)
    }
}

impl From<std::io::Error> for DnsError {
    fn from(err: std::io::Error) -> Self {
        DnsError::new(Rcode::ServFail).with_info(format!("DnsError -- {}", err))
    }
}

impl From<std::net::AddrParseError> for DnsError {
    fn from(err: std::net::AddrParseError) -> Self {
        DnsError::new(Rcode::ServFail).with_info(format!("DnsError -- {}", err))
    }
}

impl DnsError {
    pub fn new(code : Rcode) -> Self {
        Self { code, info : String::new()}
    }

    pub fn with_info(mut self, info : String) -> Self {
        self.info = info;
        self
    }
}
