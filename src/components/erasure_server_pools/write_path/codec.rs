use reed_solomon_erasure::galois_8::ReedSolomon;

use crate::types::FS3Error;

use super::bitrot::{decode_bitrot_frames, encode_bitrot_frames};

pub const DATA_SHARDS: usize = 2;
pub const PARITY_SHARDS: usize = 1;
pub const TOTAL_SHARDS: usize = DATA_SHARDS + PARITY_SHARDS;

#[derive(Debug, Clone)]
pub struct ErasureWriteResult {
    pub object_size: u64,
    pub shard_sizes: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct ErasureWriteResultWithFrames {
    pub result: ErasureWriteResult,
    pub shard_frames: Vec<Vec<u8>>,
}

pub fn encode_object_into_shards(data: &[u8]) -> Result<ErasureWriteResultWithFrames, FS3Error> {
    let codec = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS)
        .map_err(|err| FS3Error::internal(err.to_string()))?;
    let shard_len = shard_len_for(data.len());
    let mut shards = (0..TOTAL_SHARDS)
        .map(|_| vec![0u8; shard_len])
        .collect::<Vec<_>>();

    for (index, shard) in shards.iter_mut().take(DATA_SHARDS).enumerate() {
        let start = index * shard_len;
        let end = std::cmp::min(start + shard_len, data.len());
        if start < data.len() {
            shard[..end - start].copy_from_slice(&data[start..end]);
        }
    }

    codec
        .encode(&mut shards)
        .map_err(|err| FS3Error::internal(err.to_string()))?;

    let shard_frames = shards.iter().map(|shard| encode_bitrot_frames(shard)).collect();
    let shard_sizes = shards.iter().map(|shard| shard.len() as u64).collect();

    Ok(ErasureWriteResultWithFrames {
        result: ErasureWriteResult {
            object_size: data.len() as u64,
            shard_sizes,
        },
        shard_frames,
    })
}

pub fn decode_object_from_data_shards(
    shard_frames: &[Vec<u8>],
    object_size: u64,
) -> Result<Vec<u8>, FS3Error> {
    let mut combined = Vec::new();
    let shard_len = shard_len_for(object_size as usize);
    for (index, shard) in shard_frames.iter().take(DATA_SHARDS).enumerate() {
        let start = index * shard_len;
        let expected_len = if start >= object_size as usize {
            0
        } else {
            std::cmp::min(shard_len, object_size as usize - start)
        };
        let decoded = decode_bitrot_frames(shard, expected_len as u64)?;
        combined.extend_from_slice(&decoded);
    }
    combined.truncate(object_size as usize);
    Ok(combined)
}

fn shard_len_for(object_size: usize) -> usize {
    if object_size == 0 {
        0
    } else {
        object_size.div_ceil(DATA_SHARDS)
    }
}
