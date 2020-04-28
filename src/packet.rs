use byteorder::{NetworkEndian, WriteBytesExt, ReadBytesExt};
use std::io::{Result, Cursor, Read, Error, ErrorKind};

pub const HEADER_SIZE:usize = 12;
const RCODE_MASK:u8 = 0x0F;

#[derive(Debug)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Answer>,
}

impl Packet {
    pub fn serialize(self) -> Result<Vec<u8>> {
        let mut contents = Vec::new();
        let mut header = self.header.serialize()?;
        contents.append(&mut header);

        for q in self.questions.into_iter() {
            let mut bytes = q.serialize()?;
            contents.append(&mut bytes);
        }


        Ok(contents)
    }
}

#[derive(Debug)]
pub struct Header {
    pub id: u16,
    pub qr: bool,
    pub opcode: Opcode,
    pub aa: bool,
    pub tc: bool,
    pub rd: bool,
    pub ra: bool,
    pub rcode: RCode,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl Header {
    pub fn serialize(self) -> Result<Vec<u8>> {
        let mut header = Vec::new();
        header.reserve(HEADER_SIZE);
        header.write_u16::<NetworkEndian>(self.id)?;
        header.push(0);
        if !self.qr {
            header[2] |= 1 << 7;
        }
        header[2] |= (self.opcode as u8) << 2;
        if self.aa {
            header[2] |= 1 << 2;
        }
        if self.tc {
            header[2] |= 1 << 1;
        }
        if self.rd { 
            header[2] |= 1;
        }
        header.push(0);
        header[3] = self.rcode as u8;


        header.write_u16::<NetworkEndian>(self.qdcount)?;
        header.write_u16::<NetworkEndian>(self.ancount)?;
        header.write_u16::<NetworkEndian>(self.nscount)?;
        header.write_u16::<NetworkEndian>(self.arcount)?;


        Ok(header)
    }


    pub fn parse(bytes: &[u8]) -> Result<Header> {
        let mut rdr = Cursor::new(bytes);
        let id = rdr.read_u16::<NetworkEndian>()?;
        let mut control:[u8;2] = [0; 2];
        rdr.read_exact(&mut control)?;
        let qr = 0x8 & control[0] != 0;
        let opcode_bits = 0xf8 & (control[0] >> 6);
        let opcode = Opcode::from(opcode_bits);
        let rcode = RCode::from(control[1] & RCODE_MASK);
        let qdcount = rdr.read_u16::<NetworkEndian>()?;
        let ancount = rdr.read_u16::<NetworkEndian>()?;
        let nscount = rdr.read_u16::<NetworkEndian>()?;
        let arcount = rdr.read_u16::<NetworkEndian>()?;

        Ok(Header {
            id,
            qr,
            opcode,
            aa : false,
            tc : false,
            rd : false,
            ra : false,
            rcode,
            qdcount,
            ancount,
            nscount,
            arcount
        })

    }

}



#[derive(Debug)]
pub enum Opcode {
    Standard = 0,
}

impl From<u8> for Opcode {
    fn from(o: u8) -> Opcode {
        match o {
            0 => Opcode::Standard,
            o => panic!(format!("bad opcode: {}", o))
        }
    }
}



#[repr(u8)]
#[derive(Debug)]
pub enum RCode {
    NoError = 0,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused
}

impl From<u8> for RCode {
    fn from(i:u8) -> Self {
        match i {
            0 => RCode::NoError,
            1 => RCode::FormatError,
            2 => RCode::ServerFailure,
            3 => RCode::NameError,
            4 => RCode::NotImplemented,
            5 => RCode::Refused,
            _ => panic!(format!("Bad opcode: {:x}", i)),
        }
    }
}


#[derive(Debug)]
pub struct Question {
    pub name: Vec<String>,
    pub qtype: QType,
    pub qclass: QClass     
}

impl Question {
    pub fn serialize(self) -> Result<Vec<u8>> {
        let mut contents = Vec::new();
        for domain in self.name.iter() {
            let len = domain.len();
            contents.push(len as u8);
            for byte in domain.as_bytes() {
                contents.push(*byte);
            }
        }
        contents.push(0); // Null terminate name
        contents.write_u16::<NetworkEndian>(self.qtype as u16)?;
        contents.write_u16::<NetworkEndian>(self.qclass as u16)?;

        Ok(contents)
    }

    pub fn parse(bytes: &[u8]) -> Result<(Question, usize)> {
        let mut i = 0;
        let mut name = Vec::new();
        loop {
            let size = bytes[i] as usize;
            i += 1;
            if size == 0 {
                break;
            } else {
                let s = std::str::from_utf8(&bytes[i..i+size])
                    .map_err(|_| Error::new(ErrorKind::Other, "non utf"))?;
                name.push(s.to_string());
            }
            i += size;
        }

        let mut rdr = Cursor::new(&bytes[i..]);
        let qtype = QType::from(rdr.read_u16::<NetworkEndian>()?);
        let qclass = QClass::from(rdr.read_u16::<NetworkEndian>()?);

        Ok((Question { name, qtype, qclass}, i + 4))
    }


}



#[repr(u16)]
#[derive(Debug)]
pub enum QType {
    AA = 1
}

impl From<u16> for QType {
    fn from(i:u16) -> QType {
        match i {
            1 => QType::AA,
            _ => panic!(format!("Invalid QType: {}", i))
        }
    }
}

#[repr(u16)]
#[derive(Debug)]
pub enum QClass {
    Internet = 1
}

impl From<u16> for QClass {
    fn from(i:u16) -> QClass {
        match i {
            1 => QClass::Internet,
            _ => panic!(format!("Invalid QType: {}", i))
        }
    }
}


#[derive(Debug)]
pub struct Answer {
    pub atype: AType,
    pub class: QClass,
    pub ttl: u32,
    pub rdlength: u16,
    pub ip: Vec<u8>
}

const PTR_MASK:u8 = 0xC0;

impl Answer {
    pub fn parse(bytes: &[u8]) -> Result<(Answer, usize)> {
        if bytes[0] & PTR_MASK == 0 {
            return Err(Error::new(ErrorKind::Other, "bad answer packet"));
        }
        let mut rdr = Cursor::new(&bytes[2..]);
        let atype = AType::from(rdr.read_u16::<NetworkEndian>()?);
        let class = QClass::from(rdr.read_u16::<NetworkEndian>()?);
        let ttl = rdr.read_u32::<NetworkEndian>()?;
        let rdlength = rdr.read_u16::<NetworkEndian>()?;
        if rdlength != 4 {
            unimplemented!();
        }
        let mut ip = Vec::new();
        for _ in 0..4 {
            ip.push(rdr.read_u8()?);
        }

        Ok((Answer {atype, class, ttl, rdlength, ip}, 14))
    }
}

#[derive(Debug)]
#[repr(u16)]
pub enum AType {
    A = 1,
    CNAME = 0x5
}

impl From<u16> for AType {
    fn from(i:u16) -> Self {
        match i {
            1 => AType::A,
            0x5 => AType::CNAME,
            _ => panic!(format!("Bad AType: {:x}", i)),
        }
    }
}

