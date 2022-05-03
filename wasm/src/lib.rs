mod binding;
mod implementation;

use binding::JS;
use wasm_bindgen::prelude::*;
use wasmjvm_common::WasmJVMError;
use wasmjvm_vm::VM;

use crate::implementation::register;

static mut STATIC_VM: Option<VM> = None;

#[wasm_bindgen]
pub fn run() -> JsValue {
    inner_run()
}

fn inner_run() -> JsValue {
    unsafe {
        if let Some(vm) = &mut STATIC_VM {
            let result = vm.run();

            match result {
                Ok(result) => JsValue::from_str(format!("{:?}", result).as_str()),
                Err(err) => JsValue::from_str(
                    format!("{}\nError: {:?}", vm.stack_trace().unwrap(), err).as_str(),
                ),
            }
        } else {
            unreachable!()
        }
    }
}

#[wasm_bindgen]
pub fn load_jar(jar: Vec<u8>) -> JsValue {
    let jar = std::io::Cursor::new(jar);

    let result = unsafe {
        if STATIC_VM.is_none() {
            inner_boot(jar)
        } else {
            inner_load_jar(jar)
        }
    };

    match result {
        Ok(string) => JsValue::from_str(format!("{}", string).as_str()),
        Err(err) => {
            JS::error(format!("{:?}", err));
            JsValue::null()
        }
    }
}

fn inner_load_jar<B: std::io::Read + std::io::Seek>(jar: B) -> Result<String, WasmJVMError> {
    unsafe {
        if let Some(vm) = &mut STATIC_VM {
            vm.load_jars(vec![jar])?;
        } else {
            unreachable!()
        }
    };

    Ok("Loaded Jar".to_string())
}

fn inner_boot<B: std::io::Read + std::io::Seek>(jar: B) -> Result<String, WasmJVMError> {
    unsafe {
        if let Some(vm) = &mut STATIC_VM {
            vm.load_jars(vec![jar])?;
        } else {
            let mut vm = VM::new();

            vm.register_native(Box::new(wasmjvm_native::register))?;
            vm.register_native(Box::new(register))?;

            vm.boot(jar)?;

            vm.register_natives()?;

            STATIC_VM = Some(vm);
        }
    }

    Ok("Booted VM".to_string())
}
