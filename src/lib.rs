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
        value |= (current_byte[0] as i32 & 0x7F)
            .checked_shl(length * 7)
            .unwrap_or(0);
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

fn parse_json(json: &str) -> Result<ServerStatus, Box<dyn error::Error>> {
    Ok(serde_json::from_str(json)?)
    // Cast json to our custom object "ServerResponse"
}

pub fn server_status(hostname: &str, port: u16) -> Result<ServerStatus, Box<dyn error::Error>> {
    let raw_json = get_server_json(hostname, port)?;
    parse_json(&raw_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        let server_response = parse_json("{\"version\":{\"protocol\":758,\"name\":\"Velocity 1.7.2-1.18.2\"},\"players\":{\"online\":196,\"max\":150,\"sample\":[]},\"description\":{\"extra\":[{\"bold\":true,\"extra\":[{\"color\":\"aqua\",\"text\":\"E\"},{\"color\":\"aqua\",\"text\":\"a\"},{\"color\":\"aqua\",\"text\":\"r\"},{\"color\":\"aqua\",\"text\":\"t\"},{\"color\":\"aqua\",\"text\":\"h\"},{\"color\":\"green\",\"text\":\"M\"},{\"color\":\"green\",\"text\":\"C\"}],\"text\":\"\"},{\"text\":\"\\n\"},{\"color\":\"gray\",\"text\":\"Slava Ukraini!\"}],\"text\":\"\"},\"favicon\":\"data:image/png;base64,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\"}").unwrap();
        assert_eq!(server_response.players.online, 196);
        assert_eq!(server_response.players.max, 150);
        assert_eq!(server_response.version.protocol, 758);
        assert_eq!(server_response.version.name, "Velocity 1.7.2-1.18.2");
    }
}
