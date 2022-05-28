use std::sync::Arc;

use std::convert::TryInto;
use std::io::{stdout, Read, Write};
use std::net::TcpStream;

fn main() {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let server_name = "stream.cryptowat.ch".try_into().unwrap();
    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
    let mut sock = TcpStream::connect("stream.cryptowat.ch:443").unwrap();
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);

    tls.write_all(
        concat!(
            "GET /connect?apikey=AFVZF208NMBYTP8BKYRX HTTP/1.1\r\n",
            "Host: stream.cryptowat.ch\r\n",
            "Connection: Upgrade\r\n",
            "Upgrade: websocket\r\n",
            "Sec-WebSocket-Version: 13\r\n",
            "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n",
            //
            // "X-CW-API-Key: AFVZF208NMBYTP8BKYRX\r\n",
            "\r\n"
        )
        .as_bytes(),
    )
    .unwrap();

    // let mut plaintext = Vec::new();
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
