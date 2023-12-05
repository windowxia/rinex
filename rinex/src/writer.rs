//! Generic Buffered Writer, for efficient record production,
//! with integrated optionnal .gz compression
#[cfg(feature = "flate2")]
use flate2::{write::GzEncoder, Compression};
use std::fs::File;
use std::io::{Write, BufWriter};

#[derive(Debug)]
pub enum RinexWriter<W: Write> {
    /// Readable RINEX
    PlainRinex(BufWriter<W>),
    #[cfg(feature = "flate2")]
    /// Gzip compressed RINEX
    GzipRinex(BufWriter<GzEncoder<W>>),
}

impl<W: Write> RinexWriter<W> {
    /// Creates a new RinexWriter for RINEX from an input that implements [`Write`]
    pub fn new(w: W) -> Self {
        Self::PlainRinex(BufWriter::new(w))
    }
    /// Creates a new RinexWriter with seamless gzip compression,
    /// from a input that implements [`Write`]
    #[cfg(feature = "flate2")]
    #[cfg_attr(docrs, doc(cfg(feature = "flate2")))]
    pub fn new_gzip(w: W, compression_lvl: u32) -> Self {
            Self::GzipRinex(
                BufWriter::new(
                    GzEncoder::new(w, Compression::new(compression_lvl))))
    }
}

impl<W: Write> Write for RinexWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self {
            Self::PlainRinex(ref mut writer) => writer.write(buf),
            #[cfg(feature = "flate2")]
            Self::GzipRinex(ref mut writer) => writer.write(buf),
        }
    }
    fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        match self {
            Self::PlainRinex(ref mut writer) => writer.write_all(buf),
            #[cfg(feature = "flate2")]
            Self::GzipRinex(ref mut writer) => writer.write_all(buf),
        }
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        match self {
            Self::PlainRinex(ref mut writer) => writer.flush(),
            #[cfg(feature = "flate2")]
            Self::GzipRinex(ref mut writer) => writer.flush(),
        }
    }
}
