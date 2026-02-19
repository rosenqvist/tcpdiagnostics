use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use crate::metrics::{RttMetrics, compute_rtt_metrics};
use crate::protocol::{FRAME_LEN, NONCE_TAG, decode_frame, encode_frame};
use crate::report::DiagnosticReport;

pub fn run_rtt(target: &str, count: u32) -> io::Result<RttMetrics> {
    let mut stream = TcpStream::connect(target)?;

    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;

    // Disable Nagle’s algorithm.
    // Without this, TCP may buffer small writes to coalesce packets,
    // which could artificially inflate RTT measurements for small frames.
    stream.set_nodelay(true)?;

    let mut samples_ms: Vec<f64> = Vec::with_capacity(count as usize);

    // Reuse the receive buffer across iterations.
    let mut buf = [0u8; FRAME_LEN];

    for seq in 0..count {
        // Convert once per iteration
        let seq_u64 = seq as u64;

        // Construct a nonce that combines the sequence number with a fixed protocol tag.
        // The tag makes corruption or desynchronization obvious when validating replies.
        let nonce: u64 = (seq_u64 << 32) ^ NONCE_TAG;

        let frame = encode_frame(seq_u64, nonce);

        // Timing begins before the write.
        // Helps measure full round-trip time including kernel scheduling,
        // system latency, TCP stack configuration, and network latency.
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

fn measure_connect_ms(target: &str, timeout: Duration) -> io::Result<f64> {
    // connect_timeout requires a SocketAddr, so resolve it first.
    let mut addrs = target.to_socket_addrs()?;
    let addr = addrs.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "target resolved to no addresses",
        )
    })?;

    let start = Instant::now();
    let _stream = TcpStream::connect_timeout(&addr, timeout)?;
    Ok(start.elapsed().as_secs_f64() * 1000.0)
}

pub fn run_diagnose(target: &str, count: u32) -> io::Result<DiagnosticReport> {
    let mut warnings: Vec<String> = Vec::new();

    // 1) Measure the connect time
    let connect_ms = match measure_connect_ms(target, Duration::from_secs(2)) {
        Ok(ms) => Some(ms),
        Err(e) => {
            warnings.push(format!("connect failed: {}", e));
            None
        }
    };

    // 2) Run RTT if connect worked (or RTT will fail too)
    let rtt = if connect_ms.is_some() {
        match run_rtt(target, count) {
            Ok(m) => Some(m),
            Err(e) => {
                warnings.push(format!("rtt failed: {}", e));
                None
            }
        }
    } else {
        None
    };

    Ok(DiagnosticReport {
        target: target.to_string(),
        connect_ms,
        rtt,
        warnings,
    })
}
