use rustdns::{Message, Rcode};
use std::net::UdpSocket;
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
   let sock =
       match UdpSocket::bind("0.0.0.0:0") {
           Ok(s) => s,
           Err(_) => return Err (DnsError::new(Rcode::ServFail))
       };
   sock.set_read_timeout(Some(Duration::from_secs(5)));
   let root_server_ip = "198.41.0.4:53";
   match sock.connect(root_server_ip) {
       Err(_) => return Err(DnsError::new(Rcode::ServFail).with_info(format!("UDP Connection Error to Root Server: {}", root_server_ip))),
       Ok(_) => ()
   }

   let root_server_query =
       match msg.to_vec() {
           Ok(q) => q,
           Err(e)  =>  return Err(DnsError::new(Rcode::ServFail).with_info(format!("Error serializing root server query: {}", e)))
       };

   // Note from RFC 1035 2.3.4
   // UDP messages    512 octets or less
   // This is due to lower bound MTU of 576 bytes in RFC 791 Section 3.1
   // However with EDNS(0), RFC 6891 says 4096 is a good starting point
   const EDNS_RECCOMENDED_OCTETS : usize = 4096;
   if root_server_query.len() > EDNS_RECCOMENDED_OCTETS {
       return Err(DnsError::new(Rcode::ServFail).with_info(
               format!("Root server query length {} exceeds RFC 6891 reccomended length {}", root_server_query.len(), EDNS_RECCOMENDED_OCTETS)));
   }

   sock.send(&root_server_query)?;

   let mut resp = [0; EDNS_RECCOMENDED_OCTETS];
   let resp_len = sock.recv(&mut resp)?;

   let resp_msg = Message::from_slice(&resp[0..resp_len])?;
}


/// Processes a DNS query response and then sends the corresponding request to the next nameserver.
/// Returns the response from the query to the next namesever.
/// Checks to ensure that rsp is truly a DNS response, and conforms to other formatting concerns.
/// If rsp contains an answer or the returned message from the next nameserver contains an answer, then the output boolean is set to true
fn process_dns_response(rsp : &Message) -> Result<(bool, Message)> {
    // Base case where response is an answer
    if rustdns::QR::Response != rsp.qr {
        return Err(DnsError::new(Rcode::FormErr))
    }

    if let Some(ans) = dnsTools::get_answer(rsp) {

    } else if let Some(glue) = dnsTools::get_glue(rsp) {

    } else if let Some(authority) = dnsTools::get_authoritys(rsp) {

    } else {

    }


}

