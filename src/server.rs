use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::protocol::FRAME_LEN;

pub fn run_server(bind_addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(bind_addr)?;
    println!("server listening on {}", bind_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn one OS thread per client connection.
                thread::spawn(|| {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("client error: {}", e);
                    }
                });
            }
            Err(e) => {
                // Accepts can fail temporarily on Windows.
                // We log and continue instead of killing the server.
                eprintln!("accept error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let mut buf = [0u8; FRAME_LEN];

    loop {
        match stream.read_exact(&mut buf) {
            Ok(()) => {
                stream.write_all(&buf)?;
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}
