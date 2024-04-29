use crate::dnstools;
use crate::error::{DnsError, Result};
use rustdns::{
    Class, Extension, Message, Rcode,
    Resource::{A, NS},
    Type,
};
use std::net::{Ipv4Addr, UdpSocket};
use std::time::Duration;

pub fn check_format_query(msg: &Message) -> bool {
    !(rustdns::QR::Query != msg.qr || msg.questions.is_empty())
}

pub fn dispatch_query(msg: Message) -> Result<Message> {
    if msg.rd {
        recursive_resolution(msg)
    } else {
        //iterative_resolution(msg)
        Err(DnsError::new(Rcode::NXDomain))
    }
}

pub fn resolve_message_query(msg: Message) -> Result<Message> {
    if !check_format_query(&msg) {
        Err(DnsError::new(Rcode::FormErr))
    } else {
        dispatch_query(msg)
    }
}

fn print_query_response(rsp : &Message, ip : Ipv4Addr, name : Option<&str>) {
    let name_formatted = name.unwrap_or("XXX.XXX.XXX");

    println!(
            "\n\
            ----------------------QUERY START----------------------\n\
            Querying: {} {}\n\n
            {}",
            name_formatted, ip, rsp
        );
}

/*
pub fn iterative_resolution(mut msg : Message) -> Result<Message> {

}
*/

pub fn recursive_resolution(msg: Message) -> Result<Message> {
    let root_server_ip : Ipv4Addr = "198.41.0.4".parse::<Ipv4Addr>()?;
    const ROOT_SERVER_NAME : &str = "a.root-servers.net";

    let mut curr_rsp = query_name_server(root_server_ip, ROOT_SERVER_NAME, &msg)?;
    let mut ans_found = false;

    // TODO: make some sort of way to limit max iterations or loops on DNS queries
    while !ans_found {
        let (new_ans, new_rsp) = process_dns_response(&curr_rsp)?;
        ans_found = new_ans;
        curr_rsp = new_rsp;
    }

    Ok(curr_rsp)
}

pub fn query_name_server(ip: Ipv4Addr, name : &str, msg: &Message) -> Result<Message> {
    const DNS_PORT: &str = "53";

    let sock = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return Err(DnsError::new(Rcode::ServFail)),
    };
    sock.set_read_timeout(Some(Duration::from_secs(5)))?;
    let server_address = format!("{}:{}", ip, DNS_PORT);
    if sock.connect(server_address).is_err() {
        return Err(DnsError::new(Rcode::ServFail)
            .with_info(format!("UDP Connection Error to Nameserver: {}", ip)));
    }

    let nameserver_query = match msg.to_vec() {
        Ok(q) => q,
        Err(e) => {
            return Err(DnsError::new(Rcode::ServFail)
                .with_info(format!("Error serializing nameserver query: {}", e)))
        }
    };

    // Note from RFC 1035 2.3.4
    // UDP messages    512 octets or less
    // This is due to lower bound MTU of 576 bytes in RFC 791 Section 3.1
    // However with EDNS(0), RFC 6891 says 4096 is a good starting point
    const EDNS_RECCOMENDED_OCTETS: usize = 4096;
    if nameserver_query.len() > EDNS_RECCOMENDED_OCTETS {
        return Err(DnsError::new(Rcode::ServFail).with_info(format!(
            "DNS nameserver query length {} exceeds RFC 6891 reccomended length {}",
            nameserver_query.len(),
            EDNS_RECCOMENDED_OCTETS
        )));
    }

    sock.send(&nameserver_query)?;

    let mut resp = [0; EDNS_RECCOMENDED_OCTETS];
    let resp_len = sock.recv(&mut resp)?;

    let resp_msg = Message::from_slice(&resp[0..resp_len])?;

    print_query_response(&resp_msg, ip, Some(name));

    Ok(resp_msg)
}

/// Processes a DNS query response and then sends the corresponding request to the next nameserver.
/// Returns the response from the query to the next namesever.
/// Checks to ensure that rsp is truly a DNS response, and conforms to other formatting concerns.
/// If rsp contains an answer, then the output boolean is set to true
fn process_dns_response(rsp: &Message) -> Result<(bool, Message)> {
    // Base case where response is an answer
    if rustdns::QR::Response != rsp.qr {
        return Err(DnsError::new(Rcode::FormErr));
    }

    if dnstools::has_answer(rsp) {
        Ok((true, rsp.clone()))
    } else if let Some(glue) = dnstools::get_glue(rsp) {
        let mut new_msg = Message::default();

        // TODO check the class and ttl for caching and thoroughness and check for only A records
        let glue_record = glue.first().unwrap();
        let next_nameserver_ip = match &glue_record.resource {
            A(a) => a,
            _ => panic!("Couldn't find valid glue record"),
        };

        // Note that from 4.1.2 of RFC 1035 there really should only be one question due to
        // ambiguities in rcode handling.
        new_msg.questions = rsp.questions.clone();
        new_msg.add_extension(Extension {
            payload_size: 4096,
            ..Default::default()
        });

        Ok((false, query_name_server(*next_nameserver_ip, glue_record.name.as_str(), &new_msg)?))
    } else if let Some(authoritys) = dnstools::get_authoritys(rsp) {
        let mut new_msg = Message::default();
        let authority_record = authoritys.first().unwrap();
        let authority_name = match &authority_record.resource {
            NS(ns) => ns,
            _ => panic!("Couldn't find valid authority ns to redirect to"),
        };

        new_msg.add_question(authority_name.as_str(), Type::A, Class::Internet);
        new_msg.add_extension(Extension {
            payload_size: 4096,
            ..Default::default()
        });

        let authority_server_answer = resolve_message_query(new_msg.clone())?;
        let (authority_server_name, authority_server_ip) = dnstools::parse_answer_a(&authority_server_answer)?;

        // Now that we have the authority server IP, we can repeat the lookup for DNS at the same
        // authority level.
        new_msg = Message::default();
        new_msg.questions = rsp.questions.clone();
        new_msg.add_extension(Extension {
            payload_size: 4096,
            ..Default::default()
        });

        Ok((false, query_name_server(authority_server_ip, authority_server_name, &new_msg)?))
    } else {
        Err(DnsError::new(Rcode::NXDomain)
            .with_info("In resolve_message_query couldn't find next steps".to_string()))
    }
}
