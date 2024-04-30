use dash::resolver::resolve_message_query;
use dash::threadpool::{ThreadPool, ThreadPoolJob};
use rustdns::{Class, Extension, Message, Type, Rcode};
use std::time::Duration;
use std::net::{UdpSocket, SocketAddr};
use std::io::{Error, ErrorKind};

struct DashJob {
    msg : Message,
    client: SocketAddr
}

impl ThreadPoolJob for DashJob {
    fn run_job(&self) {
        match resolve_message_query(self.msg) {
            Ok(rsp) => {
                 let response_serialized = match rsp.to_vec() {
                        Ok(q) => q,
                        Err(e) => {
                            // TODO error handling
                            panic!("Response error in serializing");
                        }
                    };

                 // Bind to any currently unassigned port for sending
                 // // TODO error handling 
                 let socket_for_sending = UdpSocket::bind("0.0.0.0:0").unwrap();
                 // TODO error handling
                 socket_for_sending.send_to(&response_serialized, &self.client).unwrap();

            }
            // TODO make this return some sort of response to client
            Err(dns_error) => println!("DNS Error {} for client {}, with request: {}", dns_error, self.client, self.msg)
        }
    }
}

fn main() -> std::io::Result<()> {
    let tp = match ThreadPool::new(10, 5, 15, Duration::from_secs(5)) {
        Ok(tp) => tp,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("{}", e)))
    };

    let socket = UdpSocket::bind("0.0.0.0:50052")?;
    println!("Started Dash DNS server on port 50052");


    // Note from RFC 1035 2.3.4
    // UDP messages    512 octets or less
    // This is due to lower bound MTU of 576 bytes in RFC 791 Section 3.1
    // However with EDNS(0), RFC 6891 says 4096 is a good starting point
    const EDNS_RECCOMENDED_OCTETS: usize = 4096;
    let mut receive_buffer = [0; EDNS_RECCOMENDED_OCTETS];
    loop {
        let (rec_bytes, client) = socket.recv_from(&mut receive_buffer)?;

        let dns_request = Message::from_slice(&receive_buffer[0..rec_bytes])?;

        tp.submit_job(Box::new(DashJob { msg : dns_request, client }));
    }
}
