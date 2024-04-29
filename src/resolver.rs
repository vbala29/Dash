use rustdns::{Extension, Message, Rcode, Resource::A};
use std::net::{UdpSocket, Ipv4Addr};
use std::time::Duration;
use crate::error::{Result, DnsError};
use crate::dnsTools;

pub fn check_format_query(msg : &Message) -> bool {
    if rustdns::QR::Query != msg.qr || msg.questions.len() == 0 {
        false
    } else {
        true
    }
}

pub fn dispatch_query(msg : Message) -> Result<Message> {
    if msg.rd {
        recursive_resolution(msg)
    } else {
        iterative_resolution(msg)
    }
}

pub fn resolve_message_query(msg : Message) -> Result<Message> {
    if !check_format_query(&msg) {
        Err(DnsError::new(Rcode::FormErr))
    } else {
       dispatch_query(msg)
    }
}

pub fn iterative_resolution(mut msg : Message) -> Result<Message> {

}

pub fn recursive_resolution(mut msg: Message) -> Result<Message> {
   let root_server_ip = "198.41.0.4:53";
 }

pub fn query_name_server(ip : Ipv4Addr, msg : &Message) -> Result<Message> {
 let sock =
       match UdpSocket::bind("0.0.0.0:0") {
           Ok(s) => s,
           Err(_) => return Err (DnsError::new(Rcode::ServFail))
       };
   sock.set_read_timeout(Some(Duration::from_secs(5)));
   match sock.connect(ip.to_string()) {
       Err(_) => return Err(DnsError::new(Rcode::ServFail).with_info(format!("UDP Connection Error to Nameserver: {}", ip))),
       Ok(_) => ()
   }

   let nameserver_query =
       match msg.to_vec() {
           Ok(q) => q,
           Err(e)  =>  return Err(DnsError::new(Rcode::ServFail).with_info(format!("Error serializing nameserver query: {}", e)))
       };

   // Note from RFC 1035 2.3.4
   // UDP messages    512 octets or less
   // This is due to lower bound MTU of 576 bytes in RFC 791 Section 3.1
   // However with EDNS(0), RFC 6891 says 4096 is a good starting point
   const EDNS_RECCOMENDED_OCTETS : usize = 4096;
   if nameserver_query.len() > EDNS_RECCOMENDED_OCTETS {
       return Err(DnsError::new(Rcode::ServFail).with_info(
               format!("DNS nameserver query length {} exceeds RFC 6891 reccomended length {}", nameserver_query.len(), EDNS_RECCOMENDED_OCTETS)));
   }

   sock.send(&nameserver_query)?;

   let mut resp = [0; EDNS_RECCOMENDED_OCTETS];
   let resp_len = sock.recv(&mut resp)?;

   let resp_msg = Message::from_slice(&resp[0..resp_len])?;

   Ok(resp_msg)
}


/// Processes a DNS query response and then sends the corresponding request to the next nameserver.
/// Returns the response from the query to the next namesever.
/// Checks to ensure that rsp is truly a DNS response, and conforms to other formatting concerns.
/// If rsp contains an answer, then the output boolean is set to true
fn process_dns_response(rsp : &Message) -> Result<(bool, Message)> {
    // Base case where response is an answer
    if rustdns::QR::Response != rsp.qr {
        return Err(DnsError::new(Rcode::FormErr))
    }

    if dnsTools::has_answer(rsp) {
       Ok((true, rsp.clone()))
    } else if let Some(glue) = dnsTools::get_glue(rsp) {
        let new_msg = Message::default();

        // TODO check the class and ttl for caching and thoroughness and check for only A records
        let glue_record = glue.first().unwrap();
        let next_nameserver_ip =
            match glue_record.resource {
                A(a) => a,
                _ => panic!("Couldn't find valid glue record")
            };

        // Note that from 4.1.2 of RFC 1035 there really should only be one question due to
        // ambiguities in rcode handling.
        new_msg.questions = rsp.questions;
        new_msg.add_extension(Extension {
            payload_size: 4096,
            ..Default::default()
        });

        Ok((false, query_name_server(next_nameserver_ip, &new_msg)?))
    } else if let Some(authority) = dnsTools::get_authoritys(rsp) {

    } else {

    }


}

