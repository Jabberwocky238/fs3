use std::io;

use bytes::Bytes;
use futures::StreamExt;

use crate::types::FS3Error;
use crate::types::s3::core::BoxByteStream;

pub async fn collect_stream(mut stream: BoxByteStream) -> Result<Vec<u8>, FS3Error> {
    let mut out = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(FS3Error::from)?;
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

pub fn part_relative_path(data_dir: &str, part_number: usize) -> String {
    format!("{data_dir}/part.{part_number}")
}

pub fn to_single_chunk_stream(bytes: Vec<u8>) -> BoxByteStream {
    Box::pin(futures::stream::once(async move {
        Ok::<Bytes, io::Error>(Bytes::from(bytes))
    }))
}
