use rustls::{ClientConnection, Stream};
use std::convert::TryInto;
use std::env;

use rustls::internal::msgs::handshake;
use std::io::{stdout, BufRead, Bytes, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use tiny_ws::handshake::{http_handshake_str, parse_http_upgrade, HandshakeErr};
use tiny_ws::HttpHeader;

struct WsClient<'a, S: Read + Write> {
    stream: &'a mut S,
    read_buf: [u8; 1024],
}

impl<'a, S> WsClient<'a, S>
where
    S: 'a + Read + Write,
{
    fn new(stream: &mut S) -> WsClient<S> {
        return WsClient {
            stream: stream,
            read_buf: [0; 1024],
        };
    }
    pub fn connect(
        &mut self,
        host: &str,
        path: &str,
        headers: Vec<HttpHeader>,
    ) -> Result<(), HandshakeErr> {
        let client_handshake = http_handshake_str(host, path);
        let mut line = String::new();
        let mut buff = [0; 1024];
        if let Ok(_) = self.stream.write_all(client_handshake.as_bytes()) {
            let mut pos = 0;
            while let Ok(read) = self.stream.read(&mut buff) {
                if let Some(i) = parse_http_upgrade(&buff, pos, read, &mut line) {
                    if let Some(start_line) = line.lines().next() {
                        if "HTTP/1.1 101 Switching Protocols" != start_line {
                            return Err(HandshakeErr::WsUpgradeFailed(start_line.to_owned()));
                        }
                    }
                    for l in line.lines() {
                        println!("line={}", l);
                    }
                    break;
                }
            }
        }
        return Result::Ok(());
    }
}
fn main() {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let mut config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    env::set_var("SSLKEYLOGFILE", "/tmp/cryptowatkey.log");
    config.key_log = Arc::new(rustls::KeyLogFile::new());

    let server_name = "stream.cryptowat.ch".try_into().unwrap();
    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
    let mut sock = TcpStream::connect("stream.cryptowat.ch:443").unwrap();
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);
    let mut ws_client = WsClient::new(&mut tls);

    if let Err(e) = ws_client.connect(
        "stream.cryptowat.ch",
        "/connect?apikey=AFVZF208NMBYTP8BKYRX",
        vec![],
    ) {
        println!("error = {:?}", e);
        return;
    }

    let mut text = [0; 32];
    if let Ok(()) = tls.read_exact(&mut text) {
        stdout().write_all(&text).unwrap();
    }

    loop {
        if let Ok(_) = tls.read_exact(&mut text[0..4]) {
            stdout().write_all(&text[0..4]).unwrap();
            stdout().flush();
        } else {
            println!("err");
            break;
        }
    }
}
