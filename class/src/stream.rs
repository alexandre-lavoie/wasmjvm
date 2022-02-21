use crate::ClassError;

use std::fs;
use std::io::Read;
use std::result::Result;

#[derive(Default, Debug)]
pub struct SourceStream {
    source: Vec<u8>,
    index: usize,
}

impl SourceStream {
    pub fn from_vec(vec: &Vec<u8>) -> SourceStream {
        SourceStream {
            source: vec.clone(),
            ..Default::default()
        }
    }

    pub fn from_str(string: &str) -> SourceStream {
        SourceStream {
            source: string.to_string().into_bytes(),
            ..Default::default()
        }
    }

    pub fn from_file(path: &String) -> Result<SourceStream, ClassError> {
        if let Ok(mut file) = fs::File::open(path) {
            if let Ok(metadata) = fs::metadata(path) {
                let mut source = vec![0; metadata.len() as usize];
                if file.read(&mut source).is_ok() {
                    return Ok(SourceStream {
                        source,
                        ..Default::default()
                    });
                }
            }
        }

        Err(ClassError::FileError)
    }
}

impl Parsable<u8> for SourceStream {
    fn parse(self: &mut Self) -> Result<u8, ClassError> {
        if self.index >= self.source.len() {
            return Err(ClassError::OutOfBound);
        }

        let value = self.source[self.index];
        self.index += 1;
        Ok(value)
    }
}

impl Parsable<u16> for SourceStream {
    fn parse(self: &mut Self) -> Result<u16, ClassError> {
        if self.index + 1 >= self.source.len() {
            return Err(ClassError::OutOfBound);
        }

        let value = ((self.source[self.index] as u16) << 8) | (self.source[self.index + 1] as u16);
        self.index += 2;

        Ok(value)
    }
}

impl Parsable<u32> for SourceStream {
    fn parse(self: &mut Self) -> Result<u32, ClassError> {
        if self.index + 3 >= self.source.len() {
            return Err(ClassError::OutOfBound);
        }

        let value = (self.source[self.index] as u32) << 24
            | (self.source[self.index + 1] as u32) << 16
            | (self.source[self.index + 2] as u32) << 8
            | (self.source[self.index + 3] as u32);
        self.index += 4;

        Ok(value)
    }
}

impl<T: Streamable<T>> Parsable<T> for SourceStream {
    fn parse(self: &mut Self) -> Result<T, ClassError> {
        T::from_stream(self)
    }
}

pub trait Parsable<T> {
    fn parse(self: &mut Self) -> Result<T, ClassError>;

    fn parse_vec(self: &mut Self, count: usize) -> Result<Vec<T>, ClassError> {
        let mut output = Vec::with_capacity(count);

        for _ in 0..count {
            output.push(self.parse()?);
        }

        Ok(output)
    }
}

pub trait Streamable<T> {
    fn from_stream(stream: &mut SourceStream) -> Result<T, ClassError>;

    fn from_stream_vec(stream: &mut SourceStream, count: usize) -> Result<Vec<T>, ClassError> {
        let mut output = Vec::with_capacity(count);

        for _ in 0..count {
            output.push(Self::from_stream(stream)?);
        }

        Ok(output)
    }
}
