use highway::{HighwayHash, HighwayHasher, Key};

use crate::types::FS3Error;

const BITROT_FRAME_BLOCK_SIZE: usize = 1024 * 1024;
const MAGIC_HIGHWAYHASH256_KEY_BYTES: [u8; 32] = [
    0x4b, 0xe7, 0x34, 0xfa, 0x8e, 0x23, 0x8a, 0xcd, 0x26, 0x3e, 0x83, 0xe6, 0xbb, 0x96,
    0x85, 0x52, 0x04, 0x0f, 0x93, 0x5d, 0xa3, 0x9f, 0x44, 0x14, 0x97, 0xe0, 0x9d, 0x13,
    0x22, 0xde, 0x36, 0xa0,
];

fn highway_key() -> Key {
    Key([
        u64::from_le_bytes(MAGIC_HIGHWAYHASH256_KEY_BYTES[0..8].try_into().unwrap()),
        u64::from_le_bytes(MAGIC_HIGHWAYHASH256_KEY_BYTES[8..16].try_into().unwrap()),
        u64::from_le_bytes(MAGIC_HIGHWAYHASH256_KEY_BYTES[16..24].try_into().unwrap()),
        u64::from_le_bytes(MAGIC_HIGHWAYHASH256_KEY_BYTES[24..32].try_into().unwrap()),
    ])
}

fn hash_chunk(chunk: &[u8]) -> [u8; 32] {
    let mut hasher = HighwayHasher::new(highway_key());
    hasher.append(chunk);
    let words = hasher.finalize256();
    let mut out = [0u8; 32];
    for (index, word) in words.into_iter().enumerate() {
        out[index * 8..(index + 1) * 8].copy_from_slice(&word.to_be_bytes());
    }
    out
}

pub fn encode_bitrot_frames(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + bitrot_overhead(payload.len()));
    for chunk in payload.chunks(BITROT_FRAME_BLOCK_SIZE) {
        out.extend_from_slice(&hash_chunk(chunk));
        out.extend_from_slice(chunk);
    }
    out
}

pub fn decode_bitrot_frames(encoded: &[u8], payload_size: u64) -> Result<Vec<u8>, FS3Error> {
    let mut offset = 0usize;
    let mut out = Vec::with_capacity(payload_size as usize);
    while out.len() < payload_size as usize {
        if encoded.len().saturating_sub(offset) < 32 {
            return Err(FS3Error::bad_request("corrupt bitrot frame header"));
        }
        let expected_hash = &encoded[offset..offset + 32];
        offset += 32;

        let remaining_payload = payload_size as usize - out.len();
        let chunk_len = std::cmp::min(BITROT_FRAME_BLOCK_SIZE, remaining_payload);
        if encoded.len().saturating_sub(offset) < chunk_len {
            return Err(FS3Error::bad_request("corrupt bitrot frame payload"));
        }

        let chunk = &encoded[offset..offset + chunk_len];
        let actual_hash = hash_chunk(chunk);
        if actual_hash.as_slice() != expected_hash {
            return Err(FS3Error::bad_request("bitrot checksum mismatch"));
        }

        out.extend_from_slice(chunk);
        offset += chunk_len;
    }

    if offset != encoded.len() {
        return Err(FS3Error::bad_request("unexpected trailing bitrot bytes"));
    }

    Ok(out)
}

fn bitrot_overhead(payload_size: usize) -> usize {
    if payload_size == 0 {
        0
    } else {
        payload_size.div_ceil(BITROT_FRAME_BLOCK_SIZE) * 32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitrot_round_trip_uses_hash_prefix_per_chunk() {
        let payload = vec![7u8; BITROT_FRAME_BLOCK_SIZE + 13];
        let encoded = encode_bitrot_frames(&payload);

        assert_eq!(encoded.len(), payload.len() + 64);
        let decoded = decode_bitrot_frames(&encoded, payload.len() as u64).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn bitrot_decode_rejects_corruption() {
        let payload = b"hello world".to_vec();
        let mut encoded = encode_bitrot_frames(&payload);
        let last = encoded.len() - 1;
        encoded[last] ^= 0x01;

        let err = decode_bitrot_frames(&encoded, payload.len() as u64).unwrap_err();
        assert!(err.to_string().contains("bitrot checksum mismatch"));
    }
}
