mod implementation;

use implementation::register;
use wasmjvm_common::WasmJVMError;
use wasmjvm_native::Primitive;
use wasmjvm_vm::VM;

fn vm_eval<B: std::io::Read + std::io::Seek>(vm: &mut VM, boot_jar: B, jars: Vec<B>) -> Result<Primitive, WasmJVMError> {
    vm.register_native(Box::new(register))?;
    vm.register_native(Box::new(wasmjvm_native::register))?;

    vm.boot(boot_jar)?;

    vm.register_natives()?;

    vm.load_jars(jars)?;

    vm.run()
}

fn eval() -> Result<(), WasmJVMError> {
    let boot_jar = std::fs::File::open("./java/dist/Boot.jar").unwrap();
    let jars = vec![];

    let mut vm = VM::new();

    let result = vm_eval(&mut vm, boot_jar, jars);

    if let Ok(result) = result {
        println!("{:?}", result);
    } else if let Err(err) = result {
        println!("\n{}\n{}\n{:?}", vm.heap_trace()?, vm.stack_trace()?, err);
    } else {
        unreachable!()
    }

    Ok(())
}

fn main() {
    eval().unwrap();
}
