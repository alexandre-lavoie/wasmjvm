use wasmjvm_common::{Stream, FromData, Streamable, Parsable, WasmJVMError};

#[derive(Default, Debug)]
pub struct SourceStream {
    source: Vec<u8>,
    index: usize,
}

impl Stream<u8> for SourceStream {
    fn source(self: &Self) -> &Vec<u8> {
        &self.source
    }

    fn index(self: &Self) -> usize {
        self.index
    }

    fn index_mut(self: &mut Self) -> &mut usize {
        &mut self.index
    }
}

impl FromData for SourceStream {
    fn from_vec(vec: &Vec<u8>) -> Self {
        SourceStream {
            source: vec.clone(),
            ..Default::default()
        }
    }
}

impl<T: Streamable<SourceStream, T>> Parsable<T> for SourceStream {
    fn parse(self: &mut Self) -> Result<T, WasmJVMError> {
        T::from_stream(self)
    }
}
