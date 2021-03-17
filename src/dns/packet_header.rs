
use byteorder::{ ByteOrder, BigEndian };

use super::{OPCODE, RCODE};

mod flag {
    pub const QUERY:               u16 = 0b1000_0000_0000_0000;
    pub const OPCODE_MASK:         u16 = 0b0111_1000_0000_0000;
    pub const AUTHORITATIVE:       u16 = 0b0000_0100_0000_0000;
    pub const TRUNCATED:           u16 = 0b0000_0010_0000_0000;
    pub const RECURSION_DESIRED:   u16 = 0b0000_0001_0000_0000;
    pub const RECURSION_AVAILABLE: u16 = 0b0000_0000_1000_0000;
    // pub const AUTHENTICATED_DATA:  u16 = 0b0000_0000_0010_0000;
    // pub const CHECKING_DISABLED:   u16 = 0b0000_0000_0001_0000;
    pub const RESERVED_MASK:       u16 = 0b0000_0000_0100_0000;
    pub const RESPONSE_CODE_MASK:  u16 = 0b0000_0000_0000_1111;
}

/// Contains general information about the packet
#[derive(Debug)]
pub struct PacketHeader {
    /// The identification of the packet, must be defined when querying
    pub id: u16,
    /// Indicates if this packet is a query or a response.
    pub query: bool,
    /// Indicates the type of query in this packet
    pub opcode: OPCODE,
    /// Authoritative Answer - this bit is valid in responses,  
    /// and specifies that the responding name server is an authority for the domain name in question section.
    pub authoritative_answer: bool,
    /// TrunCation - specifies that this message was truncated due to  
    /// length greater than that permitted on the transmission channel.
    pub truncated: bool,
    /// Recursion Desired may be set in a query andis copied into the response.  
    /// If RD is set, it directs the name server to pursue the query recursively.  
    /// Recursive query support is optional.
    pub recursion_desired: bool,
    /// Recursion Available is set or cleared in a response.  
    /// It denotes whether recursive query support is available in the name server.
    pub recursion_available: bool,
    /// [RCODE](`RCODE`) indicates the response code for this packet
    pub response_code: RCODE,
    /// Indicate the number of questions in the packet
    pub questions_count: u16,
    /// Indicate the number of answers in the packet
    pub answers_count: u16,
    /// Indicate the number of name servers resource records in the packet
    pub name_servers_count: u16,
    /// Indicate the number of additional records in the packet
    pub additional_records_count: u16
}

impl PacketHeader {
    /// Creates a new header for querying
    pub fn new_query(id: u16, recursion_desired: bool) -> Self {
        Self {
            id,
            query: true,
            opcode: OPCODE::StandardQuery,
            authoritative_answer: false,
            truncated: false,
            recursion_desired,
            recursion_available: false,
            response_code: RCODE::NoError,
            additional_records_count: 0,
            answers_count: 0,
            name_servers_count: 0,
            questions_count: 0
        }
    }

    /// Parse a slice of 12 bytes into a Packet header
    pub fn parse(data: &[u8]) -> crate::Result<Self> {
        if data.len() < 12 {
            return Err(crate::SimpleDnsError::InvalidHeaderData);
        }

        let flags = BigEndian::read_u16(&data[2..4]);
        if flags & flag::RESERVED_MASK != 0 {
            return Err(crate::SimpleDnsError::InvalidHeaderData);
        }

        let header = Self {
            id: BigEndian::read_u16(&data[..2]),
            query: flags & flag::QUERY == 0,
            opcode: ((flags & flag::OPCODE_MASK)
                     >> flag::OPCODE_MASK.trailing_zeros()).into(),
            authoritative_answer: flags & flag::AUTHORITATIVE != 0,
            truncated: flags & flag::TRUNCATED != 0,
            recursion_desired: flags & flag::RECURSION_DESIRED != 0,
            recursion_available: flags & flag::RECURSION_AVAILABLE != 0,
            // authenticated_data: flags & flag::AUTHENTICATED_DATA != 0,
            // checking_disabled: flags & flag::CHECKING_DISABLED != 0,
            response_code: (flags&flag::RESPONSE_CODE_MASK).into(),
            questions_count: BigEndian::read_u16(&data[4..6]),
            answers_count: BigEndian::read_u16(&data[6..8]),
            name_servers_count: BigEndian::read_u16(&data[8..10]),
            additional_records_count: BigEndian::read_u16(&data[10..12]),
        };
        Ok(header)
    }

    /// Writes this header to a buffer of 12 bytes
    pub fn write_to(&self, buffer: &mut [u8]) {
        assert_eq!(12, buffer.len(), "Header buffer must have length of 12");

        BigEndian::write_u16(&mut buffer[0..2], self.id);
        BigEndian::write_u16(&mut buffer[2..4], self.get_flags());

        BigEndian::write_u16(&mut buffer[4..6], self.questions_count);
        BigEndian::write_u16(&mut buffer[6..8], self.answers_count);
        BigEndian::write_u16(&mut buffer[8..10], self.name_servers_count);
        BigEndian::write_u16(&mut buffer[10..12], self.additional_records_count);
    }

    fn get_flags(&self) -> u16 {
        let mut flags = 0u16;
        flags |= (self.opcode as u16)
            << flag::OPCODE_MASK.trailing_zeros();
        flags |= self.response_code as u16;
        if !self.query { flags |= flag::QUERY; }
        if self.authoritative_answer { flags |= flag::AUTHORITATIVE; }
        if self.recursion_desired { flags |= flag::RECURSION_DESIRED; }
        if self.recursion_available { flags |= flag::RECURSION_AVAILABLE; }
        if self.truncated { flags |= flag::TRUNCATED; }

        flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_example_query() {
        let header = PacketHeader {
            id: core::u16::MAX,
            query: true,
            opcode: OPCODE::StandardQuery,
            authoritative_answer: false,
            truncated: true,
            recursion_desired: true,
            recursion_available: false,
            response_code: RCODE::NoError,
            additional_records_count: 2,
            answers_count: 2,
            name_servers_count: 2,
            questions_count: 2
        };

        let mut buf = [0u8; 12];
        header.write_to(&mut buf);

        assert_eq!(b"\xff\xff\x03\x00\x00\x02\x00\x02\x00\x02\x00\x02", &buf);
    }

    #[test]
    fn parse_example_query() {
        let header = PacketHeader::parse(b"\xff\xff\x03\x00\x00\x02\x00\x02\x00\x02\x00\x02").unwrap();

            assert_eq!(core::u16::MAX, header.id);
            assert_eq!(true, header.query);
            assert_eq!(OPCODE::StandardQuery, header.opcode);
            assert_eq!(false, header.authoritative_answer);
            assert_eq!(true, header.truncated);
            assert_eq!(true, header.recursion_desired);
            assert_eq!(false, header.recursion_available);
            assert_eq!(RCODE::NoError, header.response_code);
            assert_eq!(2, header.additional_records_count);
            assert_eq!(2, header.answers_count);
            assert_eq!(2, header.name_servers_count);
            assert_eq!(2, header.questions_count);
    }
}