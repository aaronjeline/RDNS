use crate::packet::*;


pub fn create_question(domain:String) -> Packet {
    let header = Header {
        id : 0x1337,
        qr : true,
        opcode: Opcode::Standard,
        aa: false,
        tc: false,
        rd: true,
        ra: false,
        rcode: RCode::NoError,
        qdcount: 1,
        ancount: 0,
        nscount: 0,
        arcount: 0,
    };

    let parts = domain.split(".").map(|s| s.to_string()).collect();
    let q = Question { 
        name: parts,
        qtype: QType::AA,
        qclass: QClass::Internet,
    };

    Packet {
        header,
        questions: vec![q],
        answers: vec![],
    }

}
