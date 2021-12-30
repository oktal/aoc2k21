use super::{Solver, SolverError, SolverResult};

use std::collections::VecDeque;

mod hex {
    use std::num::ParseIntError;

    pub fn decode(s: &str) -> Result<Vec<u8>, ParseIntError> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect()
    }
}

mod bits {
    #[derive(Debug, Eq, PartialEq)]
    pub(super) struct Version(u8);

    #[derive(Debug, Eq, PartialEq)]
    pub(super) struct TypeId(u8);

    pub trait Primitive: Default {
        const BITS: usize;

        fn or(self, rhs: u8) -> Self;
        fn shl(self, rhs: u32) -> Self;
    }

    macro_rules! impl_primitive {
        ($t: ty) => {
            impl Primitive for $t {
                const BITS: usize = <$t>::BITS as usize;

                fn or(self, rhs: u8) -> Self {
                    self | (rhs as Self)
                }

                fn shl(self, rhs: u32) -> Self {
                    self << rhs
                }
            }
        };
    }

    impl_primitive!(u8);
    impl_primitive!(u16);

    pub struct BitReader<'a> {
        buf: &'a [u8],

        offset: usize,
    }

    impl<'a> BitReader<'a> {
        pub fn new(buf: &[u8], start_offset: usize) -> BitReader {
            BitReader {
                buf,
                offset: start_offset,
            }
        }

        pub fn consume<T: Primitive>(&mut self, count: usize) -> Option<T> {
            if count > T::BITS {
                return None;
            }

            let mut offset = self.offset;

            let mut result = T::default();
            let mut consumed_bits = 0usize;

            loop {
                let byte = self.get_byte(offset)?;

                let bit_index = offset % u8::BITS as usize;
                let remaining_bits = u8::BITS as usize - bit_index;

                let n_bits = count - consumed_bits;
                let read_bits = remaining_bits.min(n_bits);

                let value = Self::read(*byte, bit_index, read_bits);

                result = result.or(value);

                consumed_bits += read_bits;
                offset += read_bits;

                if consumed_bits == count {
                    break;
                }

                let shift_by = count - consumed_bits;
                result = result.shl(shift_by as u32);
            }

            self.offset = offset;
            Some(result)
        }

        fn get_byte(&self, bit_offset: usize) -> Option<&u8> {
            let byte_index = bit_offset / u8::BITS as usize;
            self.buf.get(byte_index)
        }

        fn read(byte: u8, bit_index: usize, count: usize) -> u8 {
            assert!(u8::BITS as usize - bit_index >= count);

            let shift = u8::BITS as usize - bit_index - count;
            let mask = ((1u16 << count) - 1) as u8;

            (byte >> shift) & mask
        }
    }

    impl Version {
        fn decode(reader: &mut BitReader<'_>) -> Option<Self> {
            Some(Self(reader.consume(3)?))
        }
    }

    impl TypeId {
        fn decode(reader: &mut BitReader<'_>) -> Option<Self> {
            Some(Self(reader.consume(3)?))
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Varint(pub u64, pub usize);

    impl Varint {
        pub fn decode(reader: &mut BitReader<'_>) -> Option<Self> {
            let mut result = 0u64;
            let mut size = 0usize;

            while let Some(group) = reader.consume::<u8>(5) {
                // There is not enough room to add 4 more bits, which means we are about to overflow
                // our integer size, return.
                if result.leading_zeros() < 4 {
                    return None;
                }

                result |= (group & 0xF) as u64;

                size += 1;

                // The top bit is not set, this is the last group, break
                if group < 0x10 {
                    return Some(Self(result, size));
                }

                result <<= 4;
            }

            // We have exhausted our slice without finding the last group
            None
        }
    }

    #[derive(Debug)]
    pub enum PacketKind {
        Sum(Vec<Packet>),
        Product(Vec<Packet>),
        Minimum(Vec<Packet>),
        Maximum(Vec<Packet>),
        Literal(Varint),
        Greater(Vec<Packet>),
        Less(Vec<Packet>),
        Equal(Vec<Packet>),
    }

    #[derive(Debug)]
    pub struct Packet {
        version: Version,

        kind: PacketKind,
    }

    impl Packet {
        pub fn version(&self) -> u8 {
            self.version.0
        }

        pub fn sub_packets(&self) -> Option<&Vec<Packet>> {
            match &self.kind {
                PacketKind::Sum(packets)
                | PacketKind::Product(packets)
                | PacketKind::Minimum(packets)
                | PacketKind::Maximum(packets)
                | PacketKind::Greater(packets)
                | PacketKind::Less(packets)
                | PacketKind::Equal(packets) => Some(packets),
                _ => None,
            }
        }

        pub fn eval(&self) -> u64 {
            match &self.kind {
                PacketKind::Sum(packets) => packets.iter().map(Self::eval).sum(),
                PacketKind::Product(packets) => packets.iter().map(Self::eval).product(),
                PacketKind::Minimum(packets) => packets.iter().map(Self::eval).min().unwrap(),
                PacketKind::Maximum(packets) => packets.iter().map(Self::eval).max().unwrap(),
                PacketKind::Literal(lit) => lit.0,
                PacketKind::Greater(packets) => {
                    let lhs = packets[0].eval();
                    let rhs = packets[1].eval();

                    if lhs > rhs {
                        1
                    } else {
                        0
                    }
                }
                PacketKind::Less(packets) => {
                    let lhs = packets[0].eval();
                    let rhs = packets[1].eval();

                    if lhs < rhs {
                        1
                    } else {
                        0
                    }
                }
                PacketKind::Equal(packets) => {
                    let lhs = packets[0].eval();
                    let rhs = packets[1].eval();

                    if lhs == rhs {
                        1
                    } else {
                        0
                    }
                }
            }
        }
    }

    const PACKET_SUM: TypeId = TypeId(0);
    const PACKET_PRODUCT: TypeId = TypeId(1);
    const PACKET_MINIMUM: TypeId = TypeId(2);
    const PACKET_MAXIMUM: TypeId = TypeId(3);
    const PACKET_LITERAL: TypeId = TypeId(4);
    const PACKET_GT: TypeId = TypeId(5);
    const PACKET_LT: TypeId = TypeId(6);
    const PACKET_EQ: TypeId = TypeId(7);

    pub fn decode(bytes: &[u8]) -> Vec<Packet> {
        let mut packets = Vec::new();
        let mut reader = BitReader::new(bytes, 0);

        while let Some(packet) = decode_packet(&mut reader) {
            packets.push(packet)
        }

        packets
    }

    fn decode_packet(reader: &mut BitReader<'_>) -> Option<Packet> {
        let version = Version::decode(reader)?;
        let type_id = TypeId::decode(reader)?;

        match type_id {
            PACKET_LITERAL => {
                let literal = Varint::decode(reader)?;
                Some(Packet {
                    version,
                    kind: PacketKind::Literal(literal),
                })
            }
            _ => {
                let length_type_id: u8 = reader.consume(1)?;
                let packets = if length_type_id == 0 {
                    let total_bits: u16 = reader.consume(15)?;
                    let end_offset = reader.offset + total_bits as usize;

                    let mut packets = Vec::new();
                    while reader.offset < end_offset {
                        packets.push(decode_packet(reader)?);
                    }

                    packets
                } else if length_type_id == 1 {
                    let packets_count: u16 = reader.consume(11)?;
                    let packets = (0..packets_count)
                        .map(|_| decode_packet(reader))
                        .collect::<Option<Vec<_>>>()?;

                    packets
                } else {
                    unreachable!();
                };

                let kind = match type_id {
                    PACKET_SUM => PacketKind::Sum(packets),
                    PACKET_PRODUCT => PacketKind::Product(packets),
                    PACKET_MINIMUM => PacketKind::Minimum(packets),
                    PACKET_MAXIMUM => PacketKind::Maximum(packets),
                    PACKET_GT => PacketKind::Greater(packets),
                    PACKET_LT => PacketKind::Less(packets),
                    PACKET_EQ => PacketKind::Equal(packets),
                    _ => return None,
                };

                Some(Packet { version, kind })
            }
        }
    }
}

struct Day16;

impl Solver for Day16 {
    fn name(&self) -> &'static str {
        "Packet Decoder"
    }

    fn solve_part1(&self, lines: Vec<String>) -> SolverResult {
        let packets = lines
            .into_iter()
            .next()
            .ok_or(SolverError::Generic("Empty packets".into()))?;

        let bytes = hex::decode(&packets).map_err(|e| SolverError::Generic(e.into()))?;
        let packets = bits::decode(bytes.as_slice());

        let mut to_traverse = packets.iter().collect::<VecDeque<_>>();
        let mut versions = Vec::new();

        while let Some(packet) = to_traverse.pop_front() {
            versions.push(packet.version() as u32);

            if let Some(sub_packets) = packet.sub_packets() {
                for packet in sub_packets {
                    to_traverse.push_back(packet)
                }
            }
        }

        let sum: u32 = versions.iter().sum();
        Ok(sum.to_string())
    }

    fn solve_part2(&self, lines: Vec<String>) -> SolverResult {
        let packets = lines
            .into_iter()
            .next()
            .ok_or(SolverError::Generic("Empty packets".into()))?;

        let bytes = hex::decode(&packets).map_err(|e| SolverError::Generic(e.into()))?;

        let packets = bits::decode(bytes.as_slice());
        let root = packets.first().ok_or(SolverError::Generic(
            "Failed to retrieve root packet".into(),
        ))?;

        Ok(root.eval().to_string())
    }

    fn test_expected(&self, part: usize) -> &'static str {
        match part {
            1 => "31",
            2 => "1",
            _ => unreachable!(),
        }
    }
}

pub(super) fn new() -> Box<dyn Solver> {
    Box::new(Day16)
}

#[cfg(test)]
mod test {
    use super::bits::*;

    #[test]
    fn should_decode_varint() {
        let bits = &[0b10111111, 0b10001010];
        let mut reader = BitReader::new(bits, 0);

        assert_eq!(Varint::decode(&mut reader), Some(Varint(2021, 3)));
    }
}
