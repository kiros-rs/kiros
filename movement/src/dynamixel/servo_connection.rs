use super::PacketManipulation;
use std::io::{Read, Write};

pub fn write_packet<W, P>(connection: &mut W, packet: P)
where
    W: Write,
    P: PacketManipulation,
{
    let pck = packet.generate().unwrap();
    connection.write(&pck).unwrap();
}

pub fn read_packet<R: Read>(connection: &mut R) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![];
    connection.read(&mut buf).unwrap();

    buf
}

pub fn read_exact_packet<R: Read>(connection: &mut R, len: usize) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![0; len];
    connection.read_exact(&mut buf).unwrap();

    buf
}
