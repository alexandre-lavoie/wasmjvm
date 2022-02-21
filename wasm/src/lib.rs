use wasm_bindgen::prelude::*;
use wasmjvm_class::{Class, SourceStream};
use wasmjvm_common::{Streamable, FromData, WasmJVMError};
use wasmjvm_vm::{VM, ObjectRef};

#[wasm_bindgen]
pub fn entry() -> String {
    match run() {
        Ok(object) => format!("{:?}", object),
        Err(err) => format!("Error: {:?}", err)
    }
}

fn run() -> Result<ObjectRef, WasmJVMError> {
    let mut vm = VM::new();
    let mut source = SourceStream::from_vec(&include_bytes!("../../tests/OnTheWeb.class").to_vec());
    let main_class = Class::from_stream(&mut source)?;
    vm.load_class_file(main_class)?;
    let result = vm.run()?;

    if let Some(object) = result {
        Ok(object)
    } else {
        Err(WasmJVMError::FileNotFound)
    }
}
