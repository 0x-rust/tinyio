use std::fmt::Write as FmtWrite;
#[derive(Debug)]
pub enum HandshakeErr {
    WsUpgradeFailed(String),
    IoErr(std::io::Error),
}
pub fn parse_http_upgrade(
    buff: &[u8],
    offset: usize,
    len: usize,
    line: &mut String,
) -> Option<usize> {
    let mut breaks = 0;
    for i in (offset..len) {
        line.push(buff[i] as char);

        if line.len() > 3 && &line[line.len() - 4..] == "\r\n\r\n" {
            return Some(i + 1);
        }
    }
    return None;
}

pub fn http_handshake_str(host: &str, path: &str) -> String {
    let mut h = String::new();
    let mut path_str = String::new();
    write!(&mut h, "Host: {}", host).unwrap();
    write!(&mut path_str, "GET {} HTTP/1.1", path).unwrap();
    let mut hand_shake = String::new();
    write!(&mut hand_shake, "GET {} HTTP/1.1\r\n", path);
    write!(&mut hand_shake, "Host: {}\r\n", host);
    write!(&mut hand_shake, "Connection: Upgrade\r\n");
    write!(&mut hand_shake, "Upgrade: websocket\r\n");
    write!(&mut hand_shake, "Sec-WebSocket-Version: 13\r\n");
    write!(
        &mut hand_shake,
        "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n"
    );
    write!(&mut hand_shake, "\r\n");
    hand_shake
}
