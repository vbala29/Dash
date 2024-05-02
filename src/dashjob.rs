use crate::lru_ttl_cache::Cache;
use crate::resolver::resolve_message_query;
use crate::threadpool::ThreadPoolJob;
use crate::dnstools::{parse_ttl_from_answer, string_of_question};
use rustdns::Message;
use rustdns::Resource::{A, AAAA, CNAME, NS, PTR};
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct DashJob {
    msg: Message,
    client: SocketAddr,
    cache: Arc<Mutex<Cache<String, Message>>>,
}

impl DashJob {
    pub fn new(
        msg: Message,
        client: SocketAddr,
        cache: Arc<Mutex<Cache<String, Message>>>,
    ) -> Self {
        DashJob { msg, client, cache }
    }
}

impl ThreadPoolJob for DashJob {
    fn run_job(&self) {
        let question_stringified = string_of_question(&self.msg).unwrap();
        let rsp;
        if let Some(cache_value) = self.cache.lock().unwrap().get(&question_stringified) {
            rsp = cache_value;
        } else {
            match resolve_message_query(&self.msg) {
                Ok(v) => {
                    rsp = v.clone();
                    self.cache.lock().unwrap().add(question_stringified, v, SystemTime::now() + parse_ttl_from_answer(&self.msg).unwrap());
                }
                // TODO make this return some sort of response to client
                Err(dns_error) => {
                    println!(
                        "{} for client {}, with request: {}",
                        dns_error, self.client, self.msg
                    );
                    return;
                }
            }
        }

        let answer_str = match &rsp.answers.first().unwrap().resource {
            A(a) => a.to_string(),
            AAAA(a) => a.to_string().to_string(),
            CNAME(c) => c.clone(),
            NS(ns) => ns.clone(),
            PTR(ptr) => ptr.clone(),
            _ => "Unhandled answer record resource type".to_string(),
        };

        // // TODO error handling
        let socket_for_sending = UdpSocket::bind("0.0.0.0:0").unwrap();
        // TODO error handling
        socket_for_sending
            .send_to(answer_str.as_bytes(), self.client)
            .unwrap();
    }
}
