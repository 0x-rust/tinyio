use std::net::TcpListener;
use std::{io, thread};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:56789").unwrap();
    for stream in listener.incoming() {
        match stream {
            Err(e) => println!("failed: {}", e),
            Ok(stream) => {
                thread::spawn(move || {
                    let mut mut_stream = stream;
                    let result = io::copy(&mut mut_stream.try_clone().unwrap(),
                                          &mut mut_stream);
                    match result {
                        Ok(size) => println!("echoed {0} bytes from {1}", size, mut_stream.peer_addr().unwrap()),
                        Err(e) => eprintln!("{}", e)
                    }
                });
            }
        }
    }
}