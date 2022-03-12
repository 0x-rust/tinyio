use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::{io, net};
use tinymux::poll;
use tinymux::poll::{Registry, ERROR, HANGUP, INVALID};

#[derive(Eq, PartialEq, Clone)]
enum Source {
    /// An event from a connected peer.
    Peer(net::SocketAddr),
    /// An event on the listening socket. Most probably a new peer connection.
    Listener,
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:56789").unwrap();
    listener.set_nonblocking(true)?;
    let mut registry = Registry::new();

    registry.register(Source::Listener, &listener, tinymux::poll::READ);
    let mut io_events = Vec::with_capacity(16);
    let mut peers: HashMap<SocketAddr, TcpStream> = HashMap::new();

    loop {
        registry.wait(&mut io_events);
        for event in &io_events {
            match event.key {
                Source::Peer(addr) if event.is_read() => {
                    let mut tcp_stream = peers.get(&addr).unwrap();
                    let size = echo_back(addr, &mut tcp_stream);
                    if size == 0 {
                        registry.unregister(&event.key);
                        if let Some(conn) = peers.remove(&addr) {
                            drop(conn);
                            println!("closing connection {0}", addr);
                        }
                    }
                }

                Source::Peer(addr) if event.is_any(HANGUP | INVALID | ERROR) => {
                    let _ = peers.remove(&addr).unwrap();
                    println!("closing connection {0}", addr);
                }
                Source::Peer(addr) => {
                    println!("unhandled event {0}", addr);
                }
                Source::Listener => {
                    loop {
                        let (mut conn, addr) = match listener.accept() {
                            Ok((conn, addr)) => (conn, addr),

                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                            Err(e) => return Err(e),
                        };
                        conn.set_nonblocking(true)?;

                        // Register the new peer using the `Peer` variant of `Source`.
                        registry.register(Source::Peer(addr), &conn, poll::READ);
                        // Save the connection to make sure it isn't dropped.
                        peers.insert(addr, conn);
                    }
                    println!("listener event ")
                }
            }
        }
    }
}

fn echo_back(addr: SocketAddr, tcp_stream: &mut &TcpStream) -> usize {
    let mut data = [0 as u8; 4096];
    let size = tcp_stream.read(&mut data).unwrap();
    tcp_stream.write(&data[0..size]).unwrap();
    return size;
}
