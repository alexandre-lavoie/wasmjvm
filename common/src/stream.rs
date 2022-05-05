use std::result::Result;

use crate::WasmJVMError;

pub trait Stream<T>: FromData + Parsable<T> {
    fn source(self: &Self) -> &Vec<T>;
    fn index(self: &Self) -> usize;
    fn index_mut(self: &mut Self) -> &mut usize;

    fn is_empty(self: &Self) -> bool {
        self.index() >= self.source().len()
    }
}

pub trait Parsable<T> {
    fn parse(self: &mut Self) -> Result<T, WasmJVMError>;

    fn parse_vec(self: &mut Self, count: usize) -> Result<Vec<T>, WasmJVMError> {
        let mut output = Vec::with_capacity(count);

        for _ in 0..count {
            output.push(self.parse()?);
        }

        Ok(output)
    }
}

impl<T: Stream<u8>> Parsable<u8> for T {
    fn parse(self: &mut Self) -> Result<u8, WasmJVMError> {
        if self.index() >= self.source().len() {
            return Err(WasmJVMError::LinkageError(format!("Class out of bound")));
        }

        let value = self.source()[self.index()];
        *self.index_mut() += 1;
        Ok(value)
    }
}

impl<T: Stream<u8>> Parsable<u16> for T {
    fn parse(self: &mut Self) -> Result<u16, WasmJVMError> {
        if self.index() + 1 >= self.source().len() {
            return Err(WasmJVMError::LinkageError(format!("Class out of bound")));
        }

        let value =
            ((self.source()[self.index()] as u16) << 8) | (self.source()[self.index() + 1] as u16);
        *self.index_mut() += 2;

        Ok(value)
    }
}

impl<T: Stream<u8>> Parsable<u32> for T {
    fn parse(self: &mut Self) -> Result<u32, WasmJVMError> {
        if self.index() + 3 >= self.source().len() {
            return Err(WasmJVMError::LinkageError(format!("Class out of bound")));
        }

        let value = (self.source()[self.index()] as u32) << 24
            | (self.source()[self.index() + 1] as u32) << 16
            | (self.source()[self.index() + 2] as u32) << 8
            | (self.source()[self.index() + 3] as u32);
        *self.index_mut() += 4;

        Ok(value)
    }
}

pub trait Streamable<T: Stream<u8>, K> {
    fn from_stream(stream: &mut T) -> Result<K, WasmJVMError>;

    fn from_stream_vec(stream: &mut T, count: usize) -> Result<Vec<K>, WasmJVMError> {
        let mut output = Vec::with_capacity(count);

        for _ in 0..count {
            output.push(Self::from_stream(stream)?);
        }

        Ok(output)
    }
}

pub trait FromData
where
    Self: Sized,
{
    fn from_vec(vec: Vec<u8>) -> Self;

    fn from_string(string: String) -> Self {
        Self::from_vec(string.into_bytes())
    }

    fn from_str(string: &str) -> Self {
        Self::from_string(string.to_string())
    }

    fn from_file<F: std::io::Read>(mut cursor: F) -> Result<Self, WasmJVMError> {
        let mut buffer = Vec::new();

        cursor.read_to_end(&mut buffer).unwrap();

        Ok(Self::from_vec(buffer))
    }
}
