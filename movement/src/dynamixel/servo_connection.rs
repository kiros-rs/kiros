use super::PacketManipulation;
use std::io::{Read, Write};

// NOTE: there isn't really any particular place to put this note so I'll just put it here
// When connected to multiple Dynamixels running different protocols, it should be possible to differentiate
// by doing one or both of these methods
// - Pulling all bytes from buffer and treating it as a complete packet
// - Attempt to parse as a protocol 2 packet before failing over to protocol 1

pub fn write_packet<W, P>(connection: &mut W, packet: &P)
where
    W: Write,
    P: PacketManipulation,
{
    let pck = packet.generate();
    connection.write(&pck).unwrap();
}

pub fn read_packet<R: Read>(connection: &mut R) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![];
    connection.read(&mut buf).unwrap();

    buf
}

// Could this return a slice?
pub fn read_exact_packet<R: Read>(connection: &mut R, len: usize) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![0; len];
    connection.read_exact(&mut buf).unwrap();

    buf
}
