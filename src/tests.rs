#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use crate::client::run_rtt;
    use crate::protocol::FRAME_LEN;

    #[test]
    fn rtt_works_on_loopback() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();

            let mut buf = [0u8; FRAME_LEN];
            loop {
                match stream.read_exact(&mut buf) {
                    Ok(()) => {
                        stream.write_all(&buf).unwrap();
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => panic!("server read error: {}", e),
                }
            }
        });

        let metrics = run_rtt(&addr.to_string(), 50).unwrap();

        assert_eq!(metrics.sent, 50);
        assert_eq!(metrics.received, 50);
        assert!(metrics.min_ms >= 0.0);
        assert!(metrics.max_ms >= metrics.min_ms);
        assert!(metrics.avg_ms >= metrics.min_ms);
        assert!(metrics.avg_ms <= metrics.max_ms);

        server_handle.join().unwrap();
    }

    #[test]
    fn client_errors_server_unreachable() {
        let addr = "127.0.0.1:59999";

        let err = crate::client::run_rtt(addr, 5).unwrap_err();

        assert!(
            matches!(
                err.kind(),
                std::io::ErrorKind::ConnectionRefused
                    | std::io::ErrorKind::TimedOut
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::AddrNotAvailable
            ),
            "unexpected error kind: {:?}, error: {}",
            err.kind(),
            err
        );
    }
}
