use dash::dashjob::DashJob;
use dash::dnserror::DnsError;
use dash::lru_ttl_cache::Cache;
use dash::threadpool::ThreadPool;
use rustdns::{Class, Extension, Message, Rcode, Type};
use std::io::{Error, ErrorKind};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    // Note from RFC 1035 2.3.4
    // UDP messages    512 octets or less
    // This is due to lower bound MTU of 576 bytes in RFC 791 Section 3.1
    // However with EDNS(0), RFC 6891 says 4096 is a good starting point
    const EDNS_RECCOMENDED_OCTETS: usize = 4096;
    let stop = Arc::new(AtomicBool::new(false));
    let stop_copy = stop.clone();

    ctrlc::set_handler(move || stop.store(true, Ordering::SeqCst))
        .expect("Error with control c logic");

    for i in 0..30 {
        std::thread::spawn(move || {
            // Sleep so the server has time to startup
            std::thread::sleep(Duration::from_millis(100));
            let mut msg = Message::default();
            msg.add_question("datatracker.ietf.org", Type::A, Class::Internet);
            msg.add_extension(Extension {
                payload_size: 4096,
                ..Default::default()
            });

            let sending_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
            sending_socket.set_nonblocking(true).unwrap();

            sending_socket
                .send_to(&msg.to_vec().unwrap(), "127.0.0.1:50051")
                .unwrap();

            let mut resp = [0; EDNS_RECCOMENDED_OCTETS];
            loop {
                match sending_socket.recv(&mut resp) {
                    Ok(_) => (),
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(2020));
                        continue;
                    }
                    Err(e) => {
                        println!(
                            "{}",
                            DnsError::new(Rcode::ServFail).with_info(format!(
                                "Issue at sock.recv in query_name_server: {}",
                                e
                            ))
                        );
                        return;
                    }
                }
                let msg_received = String::from_utf8_lossy(&resp);
                println!("Received message: {}, {} ----", i, msg_received);
                return;
            }
        });
    }

    let handle = std::thread::spawn(move || -> std::io::Result<()> {
        let tp = match ThreadPool::new(1, 0, 15, Duration::from_secs(5)) {
            Ok(tp) => tp,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("{}", e))),
        };

        let socket = UdpSocket::bind("0.0.0.0:50051")?;
        socket.set_nonblocking(true)?;
        println!("Started Dash DNS server on port 50051");

        const CACHE_CAPACITY: usize = 100;
        let cache = Arc::new(Mutex::new(Cache::<String, Message>::new(CACHE_CAPACITY)));
        Cache::start_ttl_daemon(cache.clone(), CACHE_CAPACITY);

        let mut receive_buffer = [0; EDNS_RECCOMENDED_OCTETS];
        while !stop_copy.load(Ordering::SeqCst) {
            let (rec_bytes, client) = match socket.recv_from(&mut receive_buffer) {
                Ok(s) => s,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(20));
                    continue;
                }
                Err(e) => return Err(e),
            };
            let dns_request = Message::from_slice(&receive_buffer[0..rec_bytes])?;

            tp.submit_job(Box::new(DashJob::new(dns_request, client, cache.clone())));
        }
        tp.shutdown();
        Ok(())
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
