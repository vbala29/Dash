use rustdns::{Message, Rcode};
use crate::error::{Result, DnsError};

pub fn check_format(msg : &Message) -> bool {
    if rustdns::QR::Query != msg.qr || msg.questions.len() == 0 {
        false
    } else {
        true
    }
}

pub fn dispatch_query(msg : Message) -> Result<Message> {
    if msg.rd {
        recursive_resolution(msg)
    } else {
        iterative_resolution(msg)
    }
}

pub fn resolve_message_query(msg : Message) -> Result<Message> {
    if !check_format(&msg) {
        Err (DnsError::new(Rcode::FormErr))
    } else {
       dispatch_query(msg)
    }
}

pub fn iterative_resolution(mut msg : Message) -> Result<Message> {

}

pub fn recursive_resolution(mut msg: Message) -> Result<Message> {

}

