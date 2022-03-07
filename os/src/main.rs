mod implementation;

use std::fs;

use implementation::register;
use wasmjvm_class::Primitive;
use wasmjvm_common::WasmJVMError;
use wasmjvm_vm::{VM};

fn eval() -> Result<Primitive, WasmJVMError> {
    let mut vm = VM::new();

    register(&mut vm.global_mut().native);

    let mut classes = Vec::new();
    let mut read_queue = vec!["../test/dist".to_string()];

    while !read_queue.is_empty() {
        let path = read_queue.pop().unwrap();
        let paths = fs::read_dir(path).unwrap();

        for entry in paths {
            let entry = entry.unwrap();
            let path = entry.path().to_str().unwrap().to_string();
    
            if entry.file_type().unwrap().is_dir() {
                read_queue.push(path);
            } else if entry.file_name().to_str().unwrap().ends_with(".class") {
                classes.push(path);
            }
        }
    }

    for class in &classes {
        vm.load_class_file_path(class)?
    }

    vm.run()
}

fn main() {
    let result = eval().unwrap();
}
