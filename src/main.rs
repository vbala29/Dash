use dash::resolver::resolve_message_query;
use rustdns::{Class, Extension, Message, Type};

fn main() {
    let mut msg = Message::default();
    msg.add_question("datatracker.ietf.org", Type::A, Class::Internet);
    msg.add_extension(Extension {
        payload_size: 4096,
        ..Default::default()
    });

    let rsp = resolve_message_query(msg).unwrap();
    println!("Response from lookup was: \n\n{}", rsp);
}
