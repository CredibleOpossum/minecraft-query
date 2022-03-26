use std::error;
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

mod server_object;
use server_object::ServerStatus;

const TIMEOUT: Duration = Duration::from_secs(5);
const MAX_PACKET_SIZE: u32 = 1024 * 1024 * 50; // Limit the reponse to 50MB

fn var_int_encode(num: i32) -> Vec<u8> {
    // Encodes into VarInt, https://wiki.vg/VarInt_And_VarLong
    let mut var_int = vec![];
    let mut value = num;

    while value >= 0x80 {
        var_int.push(0x80 | (value as u8));
        value >>= 7;
    }

    var_int.push(value as u8);
    var_int
}

fn var_int_read(stream: &mut TcpStream) -> Result<i32, Box<dyn error::Error>> {
    // Reads VarInt from stream, https://wiki.vg/VarInt_And_VarLong
    let mut value: i32 = 0;
    let mut length = 0;
    let mut current_byte = vec![0];

    loop {
        stream.read_exact(&mut current_byte)?;
        value |= (current_byte[0] as i32 & 0x7F) << (length * 7);
        length += 1;
        if length > 5 {
            return Err("Server's reponse had invaild VarInt".into());
        }
        if (current_byte[0] & 0x80) != 0x80 {
            break;
        }
    }
    Ok(value)
}

fn var_int_pack(data: Vec<u8>) -> Vec<u8> {
    // We are sending the length of the data encoded as VarInt, this is so minecraft knows how big the data is.
    let mut packed = var_int_encode(data.len() as i32);
    packed.extend(data); // Follow the VarInt by the data, encoding it so minecraft can use
    packed
}

fn status_packet_builder(hostname: &str, port: u16) -> Vec<u8> {
    // Builds a proper status ping, requires hostname and port because of the protocol.
    vec![
        var_int_pack(
            [
                vec![0x00, 0x00],
                var_int_pack(hostname.as_bytes().to_vec()),
                port.to_be_bytes().to_vec(),
                vec![0x01],
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        var_int_pack(vec![0x00]),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn get_server_json(hostname: &str, port: u16) -> Result<String, Box<dyn error::Error>> {
    let socket_addr = match format!("{}:{}", hostname, port).to_socket_addrs()?.next() {
        Some(socket) => socket,
        None => return Err("Failed to parse hostname".into()),
    };

    let mut stream = TcpStream::connect_timeout(&socket_addr, TIMEOUT)?; // Connect to socket

    stream.write_all(&status_packet_builder(hostname, port))?; // Send status request

    let _length = var_int_read(&mut stream)?; // Unpack length from status response (unused)
    let _id = var_int_read(&mut stream)?; // Unpack id from status response (unused)
    let string_length = var_int_read(&mut stream)?; // Unpack string length from reponse

    if string_length as u32 > MAX_PACKET_SIZE {
        return Err("Response too large".into());
    }

    let mut buffer = vec![0; string_length as usize]; // Make buffer the size of the string

    stream.read_exact(&mut buffer)?; // Read into buffer

    let json: serde_json::Value = serde_json::from_str(&String::from_utf8(buffer)?)?;

    Ok(json.to_string())
}

pub fn server_status(hostname: &str, port: u16) -> Result<ServerStatus, Box<dyn error::Error>> {
    let raw_json = get_server_json(hostname, port)?;
    let parsed: ServerStatus = serde_json::from_str(&raw_json)?; // Cast json to our custom object "ServerResponse"
    Ok(parsed)
}
