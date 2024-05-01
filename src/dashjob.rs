use crate::resolver::resolve_message_query;
use crate::threadpool::ThreadPoolJob;
use rustdns::Message;
use rustdns::Resource::{A, AAAA, CNAME, NS, PTR};
use std::net::{SocketAddr, UdpSocket};

pub struct DashJob {
    msg: Message,
    client: SocketAddr,
}

impl DashJob {
    pub fn new(msg: Message, client: SocketAddr) -> Self {
        DashJob { msg, client }
    }
}

impl ThreadPoolJob for DashJob {
    fn run_job(&self) {
        match resolve_message_query(&self.msg) {
            Ok(rsp) => {
                let answer_str = match &rsp.answers.first().unwrap().resource {
                    A(a) => a.to_string(),
                    AAAA(a) => a.to_string().to_string(),
                    CNAME(c) => c.clone(),
                    NS(ns) => ns.clone(),
                    PTR(ptr) => ptr.clone(),
                    _ => "Unhandled answer record resource type".to_string(),
                };
                // Bind to any currently unassigned port for sending
                // // TODO error handling
                let socket_for_sending = UdpSocket::bind("0.0.0.0:0").unwrap();
                // TODO error handling
                socket_for_sending
                    .send_to(answer_str.as_bytes(), self.client)
                    .unwrap();
            }
            // TODO make this return some sort of response to client
            Err(dns_error) => println!(
                "{} for client {}, with request: {}",
                dns_error, self.client, self.msg
            ),
        }
    }
}
