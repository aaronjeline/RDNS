use std::net::UdpSocket;
mod packet;
use packet::*;
mod question;
mod answer;


const DEFAULT_SERVER:&str= "8.8.8.8:53";

fn main() -> std::io::Result<()> {
    let args:Vec<_> = std::env::args().collect();
    let server:&str = args.get(1)
        .map(|s| &s[..])
        .unwrap_or(DEFAULT_SERVER);
    let socket = UdpSocket::bind("0.0.0.0:1235")?;
    println!("Connecting to {} for DNS", server);
    socket.connect(server)?;
    println!("Connected!");


    repl(&socket)?;

    Ok(())
}

fn dns_request(socket: &UdpSocket, name: String) -> std::io::Result<()> {
    let p = question::create_question(name);
    let bytes = p.serialize()?;
    socket.send(&bytes)?;
    let response = answer::recieve_ans(&socket)?;
    display_response(response);
    Ok(())
}

fn display_response(packet: Packet) {
    match packet.header.rcode {
        RCode::NoError => display_ip(packet),
        RCode::NameError => println!("Domain does not exist!"),
        _ => unimplemented!(),
    }
}

fn display_ip(packet: Packet) {
    println!("Response OK, #answers: {}", packet.header.ancount);
    for answer in packet.answers.iter() {
        print!("IPv4 Address: ");
        for (i, octet) in answer.ip.iter().enumerate() {
            if i == 3 {
                print!("{}", octet);
            } else {
                print!("{}.", octet);
            }
        }
        println!(". Time-To-Live: {}", answer.ttl);
    }
}


fn repl(socket: &UdpSocket) -> std::io::Result<()> {
    let mut domain = String::new();

    loop {
        match std::io::stdin().read_line(&mut domain) {
            Ok(_) => {
                let s = domain.clone();
                dns_request(socket, s.trim().to_string())?;
                domain = "".to_string();
            },
            Err(_) => {
                break;
            }
        }
    }

    println!("Exiting");

    Ok(())
}