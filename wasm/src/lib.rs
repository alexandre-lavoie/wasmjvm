mod implementation;
mod binding;

use binding::JS;
use wasm_bindgen::prelude::*;
use wasmjvm_class::{Class, Primitive, SourceStream};
use wasmjvm_common::{FromData, Streamable, WasmJVMError, Stream};
use wasmjvm_vm::VM;

use crate::implementation::register;

static mut STATIC_VM: Option<VM> = None;

#[wasm_bindgen]
pub fn run() -> JsValue {
    match inner_run() {
        Ok(object) => match object {
            Primitive::Boolean(value) => JsValue::from_bool(value),
            Primitive::Long(value) => JsValue::bigint_from_str(value.to_string().as_str()),
            _ => JsValue::from_str(format!("{:?}", object).as_str()),
        },
        Err(err) => JsValue::from_str(format!("Error: {:?}", err).as_str()),
    }
}

fn inner_run() -> Result<Primitive, WasmJVMError> {
    unsafe {
        if let Some(vm) = &mut STATIC_VM {
            vm.run()
        } else {
            Err(WasmJVMError::RuntimeError)
        }
    }
}

#[wasm_bindgen]
pub fn class_load(file: String) {
    inner_load(file).unwrap();
}

fn inner_load(class: String) -> Result<(), WasmJVMError> {
    JS::log(format!("{:?}", class).as_str());

    let stream = &mut SourceStream::from_vec(&class.as_bytes().to_vec());

    let class = Class::from_stream(stream)?;

    unsafe {
        if STATIC_VM.is_none() {
            let mut vm = VM::new();
            register(&mut vm.global_mut().native);
            STATIC_VM = Some(vm);
        }

        if let Some(vm) = &mut STATIC_VM {
            vm.load_class(class)?;
        }
    };

    Ok(())
}
