mod implementation;

use std::fs;

use implementation::register;
use wasmjvm_class::Class;
use wasmjvm_common::WasmJVMError;
use wasmjvm_native::{Primitive, RegisterFn};
use wasmjvm_vm::VM;

fn vm_run(vm: &mut VM, classes: Vec<Class>, main: &String) -> Result<Primitive, WasmJVMError> {
    for class in classes {
        vm.load_class(class)?;
    }

    vm.main_class_set(main)?;

    vm.register_native(Box::new(register))?;

    vm.register_native(Box::new(wasmjvm_native::register))?;

    vm.load_classes()?;

    vm.init_interface()?;

    vm.register_natives()?;

    vm.run()
}

fn eval() -> Result<(), WasmJVMError> {
    let mut class_files = Vec::new();
    let mut read_queue = vec!["/mnt/common/Github/Concordia/wasmjvm/test/dist".to_string()];

    while !read_queue.is_empty() {
        let path = read_queue.pop().unwrap();
        let paths = fs::read_dir(path).unwrap();

        for entry in paths {
            let entry = entry.unwrap();
            let path = entry.path().to_str().unwrap().to_string();

            if entry.file_type().unwrap().is_dir() {
                read_queue.push(path);
            } else if entry.file_name().to_str().unwrap().ends_with(".class") {
                class_files.push(path);
            }
        }
    }

    let classes: Vec<Class> = class_files.iter().map(|class_file| Class::from_file(class_file).unwrap()).collect();
    let main = "Main".to_string();

    let mut vm = VM::new();

    let result = vm_run(&mut vm, classes, &main);

    if let Ok(result) = result {
        println!("{:?}", result);
    } else if let Err(err) = result {
        println!("{}", vm.unwind()?);
        println!("Error: {:?}", err);
    } else {
        unreachable!()
    }

    Ok(())
}

fn main() {
    eval().unwrap()
}
