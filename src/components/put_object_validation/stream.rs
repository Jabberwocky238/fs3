use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use bytes::Bytes;
use crc32fast::Hasher as Crc32Hasher;
use futures::Stream;
use sha1::{Digest as Sha1DigestTrait, Sha1};
use sha2::Sha256;

use crate::components::put_object_validation::checksum::{
    ChecksumAlgorithm, ChecksumExpectation, ContentMd5Expectation,
};
use crate::types::FS3Error;
use crate::types::s3::core::BoxByteStream;

#[derive(Debug, Clone, Default)]
pub struct PutObjectValidationPlan {
    pub content_md5: Option<ContentMd5Expectation>,
    pub checksum: Option<ChecksumExpectation>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PutObjectValidationResult {
    pub computed_content_md5_base64: Option<String>,
    pub computed_checksum_base64: Option<String>,
}

pub struct ValidatingPutObjectStream {
    inner: BoxByteStream,
    md5: Option<md5::Context>,
    checksum: Option<ChecksumState>,
    finished: bool,
    result: PutObjectValidationResult,
}

impl ValidatingPutObjectStream {
    pub fn new(inner: BoxByteStream, plan: PutObjectValidationPlan) -> Self {
        Self {
            inner,
            md5: plan.content_md5.map(|expected| {
                let mut state = md5::Context::new();
                let _ = expected;
                state.consume([]);
                state
            }),
            checksum: plan.checksum.map(ChecksumState::new),
            finished: false,
            result: PutObjectValidationResult::default(),
        }
    }

    pub fn into_boxed_stream(self) -> BoxByteStream {
        Box::pin(self)
    }

    pub fn validation_result(&self) -> &PutObjectValidationResult {
        &self.result
    }

    fn update_digests(&mut self, chunk: &Bytes) {
        if let Some(md5) = &mut self.md5 {
            md5.consume(chunk.as_ref());
        }
        if let Some(checksum) = &mut self.checksum {
            checksum.update(chunk.as_ref());
        }
    }

    fn finalize(&mut self) -> Result<(), FS3Error> {
        if self.finished {
            return Ok(());
        }
        self.finished = true;

        if let Some(md5) = self.md5.take() {
            let digest = md5.finalize();
            let actual = STANDARD.encode(digest.0);
            self.result.computed_content_md5_base64 = Some(actual);
        }

        if let Some(checksum) = self.checksum.take() {
            let actual = checksum.finalize_base64();
            self.result.computed_checksum_base64 = Some(actual);
        }

        Ok(())
    }
}

impl Stream for ValidatingPutObjectStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                self.update_digests(&chunk);
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err))),
            Poll::Ready(None) => match self.finalize() {
                Ok(()) => Poll::Ready(None),
                Err(err) => Poll::Ready(Some(Err(io::Error::other(err.to_string())))),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

enum ChecksumState {
    Sha256 {
        expected: ChecksumExpectation,
        hasher: Sha256,
    },
    Sha1 {
        expected: ChecksumExpectation,
        hasher: Sha1,
    },
    Crc32 {
        expected: ChecksumExpectation,
        hasher: Crc32Hasher,
    },
    Crc32c {
        expected: ChecksumExpectation,
        hasher: crc32c::Crc32cHasher,
    },
}

impl ChecksumState {
    fn new(expected: ChecksumExpectation) -> Self {
        match expected.algorithm {
            ChecksumAlgorithm::Sha256 => Self::Sha256 {
                expected,
                hasher: Sha256::new(),
            },
            ChecksumAlgorithm::Sha1 => Self::Sha1 {
                expected,
                hasher: Sha1::new(),
            },
            ChecksumAlgorithm::Crc32 => Self::Crc32 {
                expected,
                hasher: Crc32Hasher::new(),
            },
            ChecksumAlgorithm::Crc32c => Self::Crc32c {
                expected,
                hasher: crc32c::Crc32cHasher::new(),
            },
        }
    }

    fn update(&mut self, chunk: &[u8]) {
        match self {
            Self::Sha256 { hasher, .. } => {
                use sha2::Digest as _;
                hasher.update(chunk);
            }
            Self::Sha1 { hasher, .. } => {
                hasher.update(chunk);
            }
            Self::Crc32 { hasher, .. } => {
                hasher.update(chunk);
            }
            Self::Crc32c { hasher, .. } => {
                hasher.update(chunk);
            }
        }
    }

    fn finalize_base64(self) -> String {
        match self {
            Self::Sha256 { hasher, .. } => STANDARD.encode({
                use sha2::Digest as _;
                hasher.finalize()
            }),
            Self::Sha1 { hasher, .. } => STANDARD.encode(hasher.finalize()),
            Self::Crc32 { hasher, .. } => STANDARD.encode(hasher.finalize().to_be_bytes()),
            Self::Crc32c { mut hasher, .. } => STANDARD.encode(hasher.finish().to_be_bytes()),
        }
    }

    #[allow(dead_code)]
    fn expected(&self) -> &ChecksumExpectation {
        match self {
            Self::Sha256 { expected, .. }
            | Self::Sha1 { expected, .. }
            | Self::Crc32 { expected, .. }
            | Self::Crc32c { expected, .. } => expected,
        }
    }
}
