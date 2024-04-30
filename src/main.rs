use dash::resolver::resolve_message_query;
use dash::threadpool::ThreadPool;
use rustdns::{Class, Extension, Message, Type};
use std::time::Duration;

fn main() {
    let tp =
        match ThreadPool::new(10, 5, 15, Duration::from_secs(5)) {
            Ok(pool) => pool,
            Err(e) => panic!("Thread pool error on creation {}", e)
        };


    let mut msg = Message::default();
    msg.add_question("datatracker.ietf.org", Type::A, Class::Internet);
    msg.add_extension(Extension {
        payload_size: 4096,
        ..Default::default()
    });

    let rsp = resolve_message_query(msg).unwrap();
    println!("Response from lookup was: \n\n{}", rsp);
}
