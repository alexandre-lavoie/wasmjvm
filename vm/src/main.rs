use wasmjvm_class::{Class, SourceStream, ClassError, Streamable};
use std::result::Result;

fn eval(path: String) -> Result<(), ClassError> {
    let mut stream = SourceStream::from_file(&path)?;
    let class = Class::from_stream(&mut stream)?;

    println!("{:?}", class);

    Ok(())
}

fn main() {
    let path = "./test/OnTheWeb.class".to_string();
    let result = eval(path);
    result.unwrap();
}
