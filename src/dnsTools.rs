use crate::error::{DnsError, Result};
use rustdns::{Message, Rcode, Record, Resource::A};

pub fn has_answer(rsp: &Message) -> bool {
    !rsp.answers.is_empty()
}

pub fn get_answer(rsp: &Message) -> Option<&Vec<Record>> {
    if !rsp.answers.is_empty() {
        Some(&rsp.answers)
    } else {
        None
    }
}

pub fn get_glue(rsp: &Message) -> Option<&Vec<Record>> {
    if !rsp.additionals.is_empty() {
        Some(&rsp.additionals)
    } else {
        None
    }
}

pub fn get_authoritys(rsp: &Message) -> Option<&Vec<Record>> {
    if !rsp.authoritys.is_empty() {
        Some(&rsp.authoritys)
    } else {
        None
    }
}

pub fn parse_answer_a(rsp: &Message) -> Result<(&str, std::net::Ipv4Addr)> {
    if !has_answer(rsp) {
        Err(DnsError::new(Rcode::ServFail))
    } else {
        let answer = rsp.answers.first().unwrap();
        match answer.resource {
            A(a) => Ok((answer.name.as_str(), a)),
            _ => Err(DnsError::new(Rcode::NXDomain)
                .with_info("Error parsing answer of record type A".to_string())),
        }
    }
}
