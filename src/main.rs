use dash::threadpool::ThreadPool;
use rustdns::{Extension, Class, Type, Message};
use std::io::{Error, ErrorKind};
use std::net::UdpSocket;
use std::time::Duration;
use dash::dashjob::DashJob;

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");
    let tp = match ThreadPool::new(10, 5, 15, Duration::from_secs(5)) {
        Ok(tp) => tp,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("{}", e))),
    };

    let socket = UdpSocket::bind("0.0.0.0:50052")?;
    println!("Started Dash DNS server on port 50052");

    std::thread::spawn(|| {
        let mut msg = Message::default();
        msg.add_question("datatracker.ietf.org", Type::A, Class::Internet);
        msg.add_extension(Extension { payload_size: 4096, ..Default::default()});

        let sending_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        for i in 0..1 {
            if i % 10 == 0 {
                std::thread::sleep(Duration::from_secs(2));
            }

            sending_socket.send_to(&msg.to_vec().unwrap(), "127.0.0.1:50052").unwrap();
        }
    });

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
