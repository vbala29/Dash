use dash::threadpool::ThreadPool;
use rustdns::Message;
use std::io::{Error, ErrorKind};
use std::net::UdpSocket;
use std::time::Duration;
use dash::dashjob::DashJob;

fn main() -> std::io::Result<()> {
    let tp = match ThreadPool::new(10, 5, 15, Duration::from_secs(5)) {
        Ok(tp) => tp,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("{}", e))),
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

        tp.submit_job(Box::new(DashJob::new(dns_request, client)));
    }
}
