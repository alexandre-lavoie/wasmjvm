mod binding;
mod implementation;

use binding::JS;
use wasm_bindgen::prelude::*;
use wasmjvm_common::WasmJVMError;
use wasmjvm_native::{Jar, Primitive};
use wasmjvm_vm::VM;

use crate::implementation::register;

static mut STATIC_VM: Option<VM> = None;

fn check_vm() -> Result<(), WasmJVMError> {
    unsafe {
        if STATIC_VM.is_none() {
            let mut vm = VM::new();

            vm.register_native(Box::new(wasmjvm_native::register))?;
            vm.register_native(Box::new(register))?;

            STATIC_VM = Some(vm);
        }
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn run() -> JsValue {
    match run_inner().await {
        Ok(result) => {
            JsValue::from_str(format!("{:?}", result).as_str())    
        },
        Err(err) => JsValue::from_str(format!("Error: {:?}", err).as_str()),
    }
}

pub async fn run_inner() -> Result<Primitive, WasmJVMError> {
    check_vm()?;

    let vm = unsafe { STATIC_VM.as_mut().unwrap() };

    vm.run().await
}

#[wasm_bindgen]
pub fn load_jar(jar: Vec<u8>) -> JsValue {
    check_vm().unwrap();

    let jar = std::io::Cursor::new(jar);

    match inner_load_jar(Jar::new(jar)) {
        Ok(string) => JsValue::from_str(format!("{}", string).as_str()),
        Err(err) => {
            JS::error(format!("{:?}", err));
            JsValue::null()
        }
    }
}

fn inner_load_jar<B: 'static + std::io::Read + std::io::Seek>(jar: Jar<B>) -> Result<String, WasmJVMError> {
    unsafe {
        if let Some(vm) = &mut STATIC_VM {
            vm.load_jar(jar)?;
        } else {
            unreachable!()
        }
    };

    Ok("Loaded Jar".to_string())
}
