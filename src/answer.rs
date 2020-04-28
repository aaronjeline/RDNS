use crate::packet::*;
use std::net::UdpSocket;
use std::io::Result;

const MAX_PACKET_SIZE: usize  = 512;

pub fn recieve_ans(socket: &UdpSocket) -> Result<Packet> {
    let mut bytes = [0; MAX_PACKET_SIZE];
    socket.recv(&mut bytes)?;
    let header = Header::parse(&bytes[0..HEADER_SIZE])?;
    let mut qs = Vec::new();
    let mut q_index = HEADER_SIZE;
    for _ in 0..header.qdcount {
        let (q,i) = Question::parse(&bytes[q_index..])?;
        qs.push(q);
        q_index += i;
    }
    let mut ans = Vec::new();
    let mut a_index = q_index;
    for _ in 0..header.ancount {
        let (a,i) = Answer::parse(&bytes[a_index..])?;
        ans.push(a);
        a_index += i;
    }

    Ok(Packet {
        header,
        questions: qs,
        answers: ans
    })
}
