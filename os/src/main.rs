mod implementation;

use implementation::register;
use wasmjvm_common::WasmJVMError;
use wasmjvm_native::{Primitive, Jar};
use wasmjvm_vm::VM;

async fn vm_eval<B: 'static + std::io::Read + std::io::Seek>(vm: &mut VM, jars: Vec<Jar<B>>) -> Result<Primitive, WasmJVMError> {
    for jar in jars {
        vm.load_jar(jar)?;
    }

    vm.register_native(Box::new(register))?;
    vm.register_native(Box::new(wasmjvm_native::register))?;

    vm.run().await
}

fn jars() -> Result<Vec<Jar<std::fs::File>>, WasmJVMError> {
    let args: Vec<String> = std::env::args().skip(1).rev().collect();

    if args.len() == 0 {
        return Err(WasmJVMError::IllegalArgumentException("Did not supply Jar to program.".to_string()));
    }

    let mut jars = Vec::new();
    for arg in args {
        let jar_path = std::env::current_dir().unwrap().join(std::path::Path::new(&arg));

        if !jar_path.exists() {
            return Err(WasmJVMError::LinkageError(format!("Could not find file {}.", jar_path.to_str().unwrap())));
        }

        jars.push(Jar::new(std::fs::File::open(jar_path).unwrap()));
    }

    Ok(jars)
}

async fn eval() -> () {
    match jars() {
        Ok(jars) => {
            let mut vm = VM::new();

            let result = vm_eval(&mut vm, jars).await;

            match result {
                Ok(_result) => {
                    // println!("{:?}", result);
                }
                Err(err) => {
                    // println!("{}", vm.heap_trace().unwrap());
                    println!("{}", vm.stack_trace().unwrap());
                    println!("{:?}", err);
                }
            }
        }
        Err(err) => {
            println!("{:?}", err);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    eval().await;

    Ok(())
}
