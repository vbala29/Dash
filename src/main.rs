use dash::dashjob::DashJob;
use dash::threadpool::ThreadPool;
use rustdns::{Class, Extension, Message, Type};
use std::io::{Error, ErrorKind};
use std::net::UdpSocket;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    // Note from RFC 1035 2.3.4
    // UDP messages    512 octets or less
    // This is due to lower bound MTU of 576 bytes in RFC 791 Section 3.1
    // However with EDNS(0), RFC 6891 says 4096 is a good starting point
    const EDNS_RECCOMENDED_OCTETS: usize = 4096;

    for i in 0..20 {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(i*100));
            let mut msg = Message::default();
            msg.add_question("datatracker.ietf.org", Type::A, Class::Internet);
            msg.add_extension(Extension {
                payload_size: 4096,
                ..Default::default()
            });

            let sending_socket = UdpSocket::bind("0.0.0.0:0").unwrap();

            sending_socket
                .send_to(&msg.to_vec().unwrap(), "127.0.0.1:50052")
                .unwrap();

            loop {
                let mut resp = [0; EDNS_RECCOMENDED_OCTETS];
                let _ = sending_socket.recv(&mut resp).unwrap();
                let msg_received = String::from_utf8_lossy(&resp);
                println!("Received message: {}, {} ----", i, msg_received);
            }
        });
    }

    let handle = std::thread::spawn(move || -> std::io::Result<()> {
        let tp = match ThreadPool::new(5, 0, 15, Duration::from_secs(5)) {
            Ok(tp) => tp,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("{}", e))),
        };

        let socket = UdpSocket::bind("0.0.0.0:50052")?;
        println!("Started Dash DNS server on port 50052");

        let mut receive_buffer = [0; EDNS_RECCOMENDED_OCTETS];
        loop {
            let (rec_bytes, client) = socket.recv_from(&mut receive_buffer)?;
            let dns_request = Message::from_slice(&receive_buffer[0..rec_bytes])?;

            tp.submit_job(Box::new(DashJob::new(dns_request, client)));
        }
    });

    match handle.join() {
        Ok(r) => match &r {
            Ok(_) => {
                println!("Shutting down server");
                r
            }
            Err(e) => {
                eprintln!("{}", e);
                r
            }
        },
        Err(_) => Err(Error::new(ErrorKind::Other, "Error in joining main loop")),
    }
}
