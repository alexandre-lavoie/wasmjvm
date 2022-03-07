use std::io::BufRead;

use wasmjvm_class::{Descriptor, MethodRef, Object, Primitive, SingleType, Type};
use wasmjvm_common::WasmJVMError;
use wasmjvm_native::{NativeEnv, NativeInterface};

macro_rules! register_method {
    ($interface: ident, $method: ident, $class: tt, $name: tt, $params: expr, $output: expr) => {
        $interface.register(
            MethodRef {
                class: $class.to_string(),
                name: $name.to_string(),
                descriptor: Descriptor::new($params, $output),
            },
            Box::new($method),
        );
    };
}

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
    register_method!(
        interface,
        string_get_internal,
        "java/lang/String",
        "getInternal",
        vec![],
        Type::Array(SingleType::Byte, 1)
    );
    register_method!(
        interface,
        string_set_internal,
        "java/lang/String",
        "setInternal",
        vec![Type::Array(SingleType::Byte, 1)],
        Type::Single(SingleType::Void)
    );
}

fn system_print(env: &mut NativeEnv) -> Primitive {
    if let [value, ..] = &env.variables()[..] {
        let value = env.reference(value).unwrap();

        if let Object::Instance(value) = value {
            let field = value.fields.get(&"<raw>".to_string()).unwrap();

            if let Primitive::String(value) = field {
                print!("{}", value);
                return Primitive::Void;
            }
        }
    }
    panic!("system_print failed.");
}

fn system_input(env: &mut NativeEnv) -> Primitive {
    let stdin = std::io::stdin();
    let line = stdin.lock().lines().next().unwrap().unwrap();
    let index = env.new_string(line).unwrap();
    let reference = Primitive::Reference(index);

    reference
}

fn string_get_internal(env: &mut NativeEnv) -> Primitive {
    if let [this, ..] = &env.variables()[..] {
        let this = env.reference(this).unwrap();

        if let Object::Instance(instance) = this {
            if let Primitive::String(value) = instance.fields.get(&"<raw>".to_string()).unwrap() {
                let raw_array: Vec<Primitive> = Vec::from_iter(value.as_bytes())
                    .iter()
                    .map(|c| Primitive::Byte(**c))
                    .collect();
                let array = Object::new_array(raw_array).unwrap();
                let index = env.alloc(array).unwrap();
                return Primitive::Reference(index);
            }
        }
    }
    panic!("string_get_internal failed.");
}

fn string_set_internal(env: &mut NativeEnv) -> Primitive {
    let variables = env.variables().clone();

    if let [this, raw, ..] = &variables[..] {
        let raw = env.reference(raw).unwrap();

        let mut raw_array: Vec<u8> = Vec::new();
        if let Object::Array(raw) = raw {
            for value in raw.iter() {
                if let Primitive::Byte(r#char) = value {
                    raw_array.push(*r#char);
                } else {
                    break;
                }
            }
        }

        let this = env.reference_mut(this).unwrap();

        if let Object::Instance(instance) = this {
            instance.fields.insert("<raw>".to_string(), Primitive::String(String::from_utf8(raw_array).unwrap()));
        }
    };

    Primitive::Void
}
