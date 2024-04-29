use rustdns::{Message, Record, Rcode, Resource::A};
use crate::error::{Result, DnsError};


pub fn has_answer(rsp: &Message) -> bool {
    rsp.answers.len() > 0
}


pub fn get_answer(rsp : &Message) -> Option<&Vec<Record>> {
    if rsp.answers.len() > 0 {
        Some(&rsp.answers)
    } else {
        None
    }
}

pub fn get_glue(rsp : &Message) -> Option<&Vec<Record>> {
    if rsp.additionals.len() > 0 {
        Some(&rsp.additionals)
    } else {
        None
    }
}

pub fn get_authoritys(rsp : &Message) -> Option<&Vec<Record>> {
    if rsp.authoritys.len() > 0 {
        Some(&rsp.authoritys)
    } else {
        None
    }
}

pub fn parse_answer_a(rsp : &Message) -> Result<std::net::Ipv4Addr> {
    if !has_answer(rsp) {
        Err(DnsError::new(Rcode::ServFail))
    } else {
        let answer = rsp.answers.first().unwrap();
        match answer.resource {
            A(a) => Ok(a),
            _ => Err(DnsError::new(Rcode::NXDomain))
        }
    }
}
