use crate::dnserror::{DnsError, Result};
use rustdns::{Message, Rcode, Record, Resource::A};
use std::time::Duration;

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
        Err(DnsError::new(Rcode::ServFail)
            .with_info("Expected an answer but didn't get one".to_string()))
    } else {
        let answer = rsp.answers.first().unwrap();
        match answer.resource {
            A(a) => Ok((answer.name.as_str(), a)),
            _ => Err(DnsError::new(Rcode::NXDomain)
                .with_info("Error parsing answer of record type A".to_string())),
        }
    }
}

pub fn string_of_question(rsp: &Message) -> Result<String> {
    let question = rsp.questions.first().expect("No questions present");
    Ok(format!("{} {} {}", question.name, question.r#type, question.class))
}

pub fn parse_ttl_from_answer(rsp: &Message) -> Result<Duration> {
    Ok(rsp.answers.first().expect("No answers available").ttl)
}
