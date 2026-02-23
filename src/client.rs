use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use crate::metrics::{RttMetrics, compute_rtt_metrics};
use crate::protocol::{FRAME_LEN, NONCE_TAG, decode_frame, encode_frame};
use crate::report::DiagnosticReport;

fn run_rtt_on_stream(stream: &mut TcpStream, count: u32) -> io::Result<RttMetrics> {
    let mut samples_ms: Vec<f64> = Vec::with_capacity(count as usize);

    // Reuse the receive buffer across iterations.
    let mut buf = [0u8; FRAME_LEN];

    for seq in 0..count {
        let seq_u64 = seq as u64;

        // Construct a nonce that combines the sequence number with a fixed protocol tag.
        let nonce: u64 = (seq_u64 << 32) ^ NONCE_TAG;

        let frame = encode_frame(seq_u64, nonce);

        // Timing begins before the write.
        let start = Instant::now();
        stream.write_all(&frame)?;

        stream.read_exact(&mut buf)?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let (reply_seq, reply_nonce) = decode_frame(&buf);
        if reply_seq != seq_u64 || reply_nonce != nonce {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "mismatched reply: expected seq={},nonce={}, got seq={},nonce={}",
                    seq_u64, nonce, reply_seq, reply_nonce
                ),
            ));
        }

        samples_ms.push(elapsed);
    }

    Ok(compute_rtt_metrics(&samples_ms, count))
}

pub fn run_rtt(target: &str, count: u32) -> io::Result<RttMetrics> {
    let mut stream = TcpStream::connect(target)?;

    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;

    // disable nagle
    stream.set_nodelay(true)?;

    run_rtt_on_stream(&mut stream, count)
}

fn measure_connect_ms(target: &str, timeout: Duration) -> io::Result<(TcpStream, f64)> {
    // connect_timeout requires a SocketAddr, so resolve it first.
    let mut addrs = target.to_socket_addrs()?;
    let addr = addrs.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "target resolved to no addresses",
        )
    })?;

    let start = Instant::now();
    let stream = TcpStream::connect_timeout(&addr, timeout)?;
    let ms = start.elapsed().as_secs_f64() * 1000.0;

    Ok((stream, ms))
}

pub fn run_diagnose(target: &str, count: u32) -> io::Result<DiagnosticReport> {
    let mut warnings: Vec<String> = Vec::new();

    let (mut stream, connect_ms) = match measure_connect_ms(target, Duration::from_secs(2)) {
        Ok((s, ms)) => (s, Some(ms)),
        Err(e) => {
            warnings.push(format!("connect failed: {}", e));
            return Ok(DiagnosticReport {
                target: target.to_string(),
                connect_ms: None,
                rtt: None,
                warnings,
            });
        }
    };

    // Configure the *same* stream we just connected with.
    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(2))) {
        warnings.push(format!("set_read_timeout failed: {}", e));
    }
    if let Err(e) = stream.set_write_timeout(Some(Duration::from_secs(2))) {
        warnings.push(format!("set_write_timeout failed: {}", e));
    }
    if let Err(e) = stream.set_nodelay(true) {
        warnings.push(format!("set_nodelay failed: {}", e));
    }

    // Run RTT on the same TCP connection.
    let rtt = match run_rtt_on_stream(&mut stream, count) {
        Ok(m) => Some(m),
        Err(e) => {
            warnings.push(format!("rtt failed: {}", e));
            None
        }
    };

    Ok(DiagnosticReport {
        target: target.to_string(),
        connect_ms,
        rtt,
        warnings,
    })
}
