//! Buffered Reader wrapper, for efficient data reading
//! and integrated .gz decompression.
#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufReader, Read}; // Seek, SeekFrom};

#[derive(Debug)]
#[cfg(feature = "lzw")]
struct LzwDecoder<R: Read> {
    inner: Vec<u8>,
    stream: R,
}

#[cfg(feature = "lzw")]
impl<R: Read> LzwDecoder<R> {
    fn new(stream: R) -> Self {
        Self {
            stream,
            inner: Vec::with_capacity(128),
        }
    }
}

#[cfg(feature = "lzw")]
impl<R: Read> std::io::Read for LzwDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.stream.read(buf)
    }
}

#[cfg(feature = "lzw")]
impl<R: Read> std::io::BufRead for LzwDecoder<R> {
    fn fill_buf(&mut self) -> Result<&[u8], std::io::Error> {
        Ok(Default::default())
    }
    fn consume(&mut self, s: usize) {}
}

#[derive(Debug)]
pub enum BufferedReader {
    /// Readable `RINEX`
    Plain(BufReader<File>),
    /// gzip compressed RINEX
    #[cfg(feature = "flate2")]
    Gzip(BufReader<GzDecoder<File>>),
    /// z compressed RINEX
    #[cfg(feature = "lzw")]
    Z(BufReader<LzwDecoder<File>>),
}

impl BufferedReader {
    /// Builds a new BufferedReader for efficient file interation,
    /// with possible .gz decompression
    pub fn new(path: &str) -> std::io::Result<Self> {
        let f = File::open(path)?;
        if path.ends_with(".gz") {
            // --> Gzip compressed
            #[cfg(feature = "flate2")]
            {
                Ok(Self::Gzip(BufReader::new(GzDecoder::new(f))))
            }
            #[cfg(not(feature = "flate2"))]
            {
                panic!(".gzip compressed files require --flate2 feature")
            }
        } else if path.ends_with(".Z") {
            // --> Z compressed
            #[cfg(feature = "lzw")]
            {
                Ok(Self::Z(BufReader::new(LzwDecoder::new(f))))
            }
            #[cfg(not(feature = "lzw"))]
            {
                panic!(".Z compressed files require --flate2 feature")
            }
        } else {
            // Assumes no extra compression
            Ok(Self::Plain(BufReader::new(f)))
        }
    }
}

impl std::io::Read for BufferedReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self {
            Self::Plain(ref mut h) => h.read(buf),
            #[cfg(feature = "lzw")]
            Self::Z(ref mut h) => h.read(buf),
            #[cfg(feature = "flate2")]
            Self::Gzip(ref mut h) => h.read(buf),
        }
    }
}

impl std::io::BufRead for BufferedReader {
    fn fill_buf(&mut self) -> Result<&[u8], std::io::Error> {
        match self {
            Self::Plain(ref mut bufreader) => bufreader.fill_buf(),
            #[cfg(feature = "lzw")]
            Self::Z(ref mut bufreader) => bufreader.fill_buf(),
            #[cfg(feature = "flate2")]
            Self::Gzip(ref mut bufreader) => bufreader.fill_buf(),
        }
    }
    fn consume(&mut self, s: usize) {
        match self {
            Self::Plain(ref mut bufreader) => bufreader.consume(s),
            #[cfg(feature = "lzw")]
            Self::Z(ref mut bufreader) => bufreader.consume(s),
            #[cfg(feature = "flate2")]
            Self::Gzip(ref mut bufreader) => bufreader.consume(s),
        }
    }
}
