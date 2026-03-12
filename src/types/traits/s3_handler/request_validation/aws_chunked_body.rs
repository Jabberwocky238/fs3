use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use bytes::{Buf, Bytes, BytesMut};
use futures::Stream;

use crate::types::FS3Error;
use crate::types::s3::core::BoxByteStream;
use crate::types::traits::s3_handler::request_validation::chunked::AwsChunkedUpload;
use crate::types::traits::s3_handler::request_validation::trailer::{
    DeclaredTrailerNames, ParsedTrailerHeaders, parse_declared_trailer_names, parse_trailer_block,
};

#[derive(Debug, Clone, Default)]
pub struct AwsChunkedBodyDecodeResult {
    inner: Arc<Mutex<AwsChunkedBodyDecodeState>>,
}

#[derive(Debug, Default)]
struct AwsChunkedBodyDecodeState {
    decoded_bytes: u64,
    parsed_trailers: Option<ParsedTrailerHeaders>,
}

impl AwsChunkedBodyDecodeResult {
    fn new() -> Self {
        Self::default()
    }

    fn set_decoded_bytes(&self, decoded_bytes: u64) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.decoded_bytes = decoded_bytes;
        }
    }

    fn set_trailers(&self, parsed_trailers: ParsedTrailerHeaders) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.parsed_trailers = Some(parsed_trailers);
        }
    }

    pub fn decoded_bytes(&self) -> u64 {
        self.inner
            .lock()
            .map(|inner| inner.decoded_bytes)
            .unwrap_or_default()
    }

    pub fn parsed_trailers(&self) -> Option<ParsedTrailerHeaders> {
        self.inner
            .lock()
            .ok()
            .and_then(|inner| inner.parsed_trailers.clone())
    }
}

pub struct DecodedAwsChunkedStream {
    inner: BoxByteStream,
    pending: BytesMut,
    eof: bool,
    done: bool,
    current_chunk_remaining: usize,
    decoded_bytes: u64,
    expected_decoded_length: u64,
    declared_trailers: Option<DeclaredTrailerNames>,
    result: AwsChunkedBodyDecodeResult,
}

impl DecodedAwsChunkedStream {
    pub fn new(
        inner: BoxByteStream,
        upload: &AwsChunkedUpload,
    ) -> Result<(Self, AwsChunkedBodyDecodeResult), FS3Error> {
        let result = AwsChunkedBodyDecodeResult::new();
        let declared_trailers = parse_declared_trailer_names(upload.trailer.as_deref())?;
        Ok((
            Self {
                inner,
                pending: BytesMut::new(),
                eof: false,
                done: false,
                current_chunk_remaining: 0,
                decoded_bytes: 0,
                expected_decoded_length: upload.decoded_content_length,
                declared_trailers,
                result: result.clone(),
            },
            result,
        ))
    }

    pub fn into_boxed_stream(self) -> BoxByteStream {
        Box::pin(self)
    }

    fn parse_next_frame(&mut self) -> Result<Option<Bytes>, FS3Error> {
        if self.done {
            return Ok(None);
        }

        if self.current_chunk_remaining > 0 {
            if self.pending.len() < self.current_chunk_remaining + 2 {
                return Ok(None);
            }
            let chunk = self.pending.split_to(self.current_chunk_remaining).freeze();
            if &self.pending[..2] != b"\r\n" {
                return Err(FS3Error::bad_request(
                    "Invalid aws-chunked chunk terminator",
                ));
            }
            self.pending.advance(2);
            self.current_chunk_remaining = 0;
            self.decoded_bytes = self.decoded_bytes.saturating_add(chunk.len() as u64);
            self.result.set_decoded_bytes(self.decoded_bytes);
            return Ok(Some(chunk));
        }

        let Some(header_end) = find_bytes(&self.pending, b"\r\n") else {
            return Ok(None);
        };
        let header = self.pending.split_to(header_end);
        self.pending.advance(2);

        let header = std::str::from_utf8(&header)
            .map_err(|_| FS3Error::bad_request("Invalid aws-chunked header"))?;
        let size_text = header.split(';').next().unwrap_or_default().trim();
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| FS3Error::bad_request("Invalid aws-chunked chunk size"))?;

        if size == 0 {
            let Some(trailer_end) = find_bytes(&self.pending, b"\r\n\r\n") else {
                if self.eof {
                    return Err(FS3Error::bad_request("Incomplete aws-chunked trailer"));
                }
                return Ok(None);
            };
            let trailer_block = self.pending.split_to(trailer_end);
            self.pending.advance(4);
            let parsed = parse_trailer_block(&trailer_block, self.declared_trailers.as_ref())?;
            self.result.set_trailers(parsed);
            if self.decoded_bytes != self.expected_decoded_length {
                return Err(FS3Error::bad_request(
                    "Decoded content length does not match x-amz-decoded-content-length",
                ));
            }
            self.done = true;
            return Ok(None);
        }

        self.current_chunk_remaining = size;
        self.parse_next_frame()
    }
}

impl Stream for DecodedAwsChunkedStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.parse_next_frame() {
                Ok(Some(chunk)) => return Poll::Ready(Some(Ok(chunk))),
                Ok(None) if self.done => return Poll::Ready(None),
                Ok(None) if self.eof => {
                    return Poll::Ready(Some(Err(io::Error::other(
                        "Incomplete aws-chunked stream",
                    ))));
                }
                Ok(None) => {}
                Err(err) => return Poll::Ready(Some(Err(io::Error::other(err.to_string())))),
            }

            match self.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => self.pending.extend_from_slice(&chunk),
                Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                Poll::Ready(None) => self.eof = true,
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

fn find_bytes(buf: &BytesMut, needle: &[u8]) -> Option<usize> {
    buf.windows(needle.len())
        .position(|window| window == needle)
}
