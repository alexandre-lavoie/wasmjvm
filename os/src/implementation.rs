use std::io::BufRead;

use wasmjvm_class::{Descriptor, MethodRef, SingleType, Type};
use wasmjvm_native::{register_method, NativeEnv, NativeInterface, Primitive, RustObject};

pub fn register(interface: &mut NativeInterface) {
    register_method!(
        interface,
        system_print,
        "java/io/PrintStream",
        "print",
        vec![Type::Single(SingleType::Object(
            "java/lang/String".to_string()
        ))],
        Type::Single(SingleType::Void)
    );
    register_method!(
        interface,
        system_input,
        "java/io/InputStream",
        "input",
        vec![],
        Type::Single(SingleType::Object("java/lang/String".to_string()))
    );
}

fn system_print(env: &mut NativeEnv) -> Primitive {
    if let [value, ..] = &env.variables()[..] {
        let value = env.reference(value).unwrap();

        if let RustObject::String(value) = value.inner() {
            print!("{}", value);
            return Primitive::Void;
        };
    }
    panic!("system_print failed.");
}

fn system_input(env: &mut NativeEnv) -> Primitive {
    println!("");

    let stdin = std::io::stdin();
    let line = stdin.lock().lines().next().unwrap().unwrap();
    let index = env.new_string(line).unwrap();
    let reference = Primitive::Reference(index);

    reference
}
