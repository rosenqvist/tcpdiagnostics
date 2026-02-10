pub const FRAME_LEN: usize = 16;

pub fn encode_frame(seq: u64, nonce: u64) -> [u8; FRAME_LEN] {
    let mut buf = [0u8; FRAME_LEN];

    buf[0..8].copy_from_slice(&seq.to_be_bytes());
    buf[8..16].copy_from_slice(&nonce.to_be_bytes());

    buf
}

pub fn decode_frame(buf: &[u8; FRAME_LEN]) -> (u64, u64) {
    let seq = u64::from_be_bytes(buf[0..8].try_into().unwrap());
    let nonce = u64::from_be_bytes(buf[8..16].try_into().unwrap());

    (seq, nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_roundtrip() {
        let seq = 42u64;
        let nonce = 0xAABBCCDDu64; //0xAA 0xBB 0xCC 0xDD

        let buf = encode_frame(seq, nonce);
        let (decoded_seq, decoded_nonce) = decode_frame(&buf);

        assert_eq!(seq, decoded_seq);
        assert_eq!(nonce, decoded_nonce);
    }
}
