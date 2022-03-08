use crate::binding::JS;

use wasmjvm_class::{Descriptor, MethodRef, SingleType, Type};
use wasmjvm_native::{Object, Primitive, NativeEnv, NativeInterface, register_method, RustObject};

pub fn register(interface: &mut NativeInterface) {
    register_method!(
        interface,
        system_print,
        "java/lang/System",
        "print",
        vec![Type::Single(SingleType::Object(
            "java/lang/String".to_string()
        ))],
        Type::Single(SingleType::Void)
    );
    register_method!(
        interface,
        system_input,
        "java/lang/System",
        "input",
        vec![],
        Type::Single(SingleType::Object("java/lang/String".to_string()))
    );
}

fn system_print(env: &mut NativeEnv) -> Primitive {
    if let [value, ..] = &env.variables()[..] {
        let object = env.reference(value).unwrap();

        if let RustObject::String(value) = object.inner() {
            JS::log(format!("{}", value));

            Primitive::Void
        } else {
            todo!()
        }
    } else {
        todo!()
    }
}

fn system_input(env: &mut NativeEnv) -> Primitive {
    let line = JS::prompt();
    let index = env.new_string(line).unwrap();
    let reference = Primitive::Reference(index);

    reference
}
