mod binding;
mod implementation;

use binding::JS;
use wasm_bindgen::prelude::*;
use wasmjvm_class::{Class, SourceStream};
use wasmjvm_common::{FromData, Stream, Streamable, WasmJVMError};
use wasmjvm_native::Primitive;
use wasmjvm_vm::VM;

use crate::implementation::register;

static mut STATIC_VM: Option<VM> = None;

#[wasm_bindgen]
pub fn run() -> JsValue {
    inner_run()
}

fn vm_run(vm: &mut VM) -> Result<Primitive, WasmJVMError> {
    vm.load_classes()?;

    vm.init_interface()?;

    vm.register_natives()?;

    vm.run()
}

fn inner_run() -> JsValue {
    unsafe {
        if let Some(vm) = &mut STATIC_VM {
            let result =  vm_run(vm);

            match result {
                Ok(result) => JsValue::from_str(format!("{:?}", result).as_str()),
                Err(err) => {
                    JsValue::from_str(format!("{}\nError: {:?}", vm.unwind().unwrap(), err).as_str())
                }
            }
        } else {
            unreachable!()
        }
    }
}

#[wasm_bindgen]
pub fn class_load(file: Vec<u8>) -> JsValue {
    match inner_load(&file) {
        Ok(string) => {
            JsValue::from_str(format!("{}", string).as_str())
        }
        Err(err) => {
            JS::error(format!("{:?}", err));
            JsValue::null()
        }
    }
}

fn inner_load(class: &Vec<u8>) -> Result<String, WasmJVMError> {
    let stream = &mut SourceStream::from_vec(class);

    let class = Class::from_stream(stream)?;

    let class_raw = format!("{:?}", class.this_class());

    unsafe {
        if STATIC_VM.is_none() {
            let mut vm = VM::new();
            vm.register_native(Box::new(wasmjvm_native::register))?;
            vm.register_native(Box::new(register))?;
            STATIC_VM = Some(vm);
        }

        if let Some(vm) = &mut STATIC_VM {
            vm.load_class(class)?;
        } else {
            unreachable!()
        }
    };

    Ok(class_raw)
}

#[wasm_bindgen]
pub fn main_class(cls: String) {
    unsafe {
        if let Some(vm) = &mut STATIC_VM {
            vm.main_class_set(&cls).unwrap();
        } else {
            unreachable!()
        }
    }
}
