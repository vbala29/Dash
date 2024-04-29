use rustdns::{Message, Record};


pub fn get_answer(rsp : &Message) -> Option<&Vec<Record>> {
    if rsp.answers.len() > 0 {
        Some(&rsp.answers)
    } else {
        None
    }
}

pub fn get_glue(rsp : &Message) -> Option<&Vec<Record>> {
    if rsp.additionals.len() > 0 {
        Some(&rsp.additionals)
    } else {
        None
    }
}

pub fn get_authoritys(rsp : &Message) -> Option<&Vec<Record>> {
    if rsp.authoritys.len() > 0 {
        Some(&rsp.authoritys)
    } else {
        None
    }
}
