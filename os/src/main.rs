use wasmjvm_common::WasmJVMError;
use wasmjvm_vm::{VM};

fn eval() -> Result<(), WasmJVMError> {
    let mut vm = VM::new();
    vm.load_class_file_path(&"../tests/OnTheWeb.class".to_string())?;
    let result = vm.run()?;

    println!("{:?}", result);

    Ok(())
}

fn main() {
    eval().unwrap();
}
