use rustdns::Message;
use crate::threadpool::ThreadPoolJob;
use std::net::{SocketAddr, UdpSocket};
use crate::resolver::resolve_message_query;

pub struct DashJob {
    msg: Message,
    client: SocketAddr,
}

impl DashJob {
    pub fn new(msg : Message, client : SocketAddr) -> Self {
        DashJob { msg, client }
    }
}

impl ThreadPoolJob for DashJob {
    fn run_job(&self) {
        match resolve_message_query(&self.msg) {
            Ok(rsp) => {
                let response_serialized = match rsp.to_vec() {
                    Ok(q) => q,
                    Err(_) => {
                        // TODO error handling
                        panic!("Response error in serializing");
                    }
                };

                // Bind to any currently unassigned port for sending
                // // TODO error handling
                let socket_for_sending = UdpSocket::bind("0.0.0.0:0").unwrap();
                // TODO error handling
                socket_for_sending
                    .send_to(&response_serialized, self.client)
                    .unwrap();
            }
            // TODO make this return some sort of response to client
            Err(dns_error) => println!(
                "DNS Error {} for client {}, with request: {}",
                dns_error, self.client, self.msg
            ),
        }
    }
}
