use wasmjvm_class::{Descriptor, MethodRef, SingleType, Type};
use wasmjvm_common::WasmJVMError;
use crate::{NativeEnv, NativeInterface, Object, Primitive, RustObject};

#[macro_export]
macro_rules! register_method {
    ($interface: ident, $method: ident, $class: tt, $name: tt, $params: expr, $output: expr) => {
        $interface
            .register(
                MethodRef::new(
                    $class.to_string(),
                    $name.to_string(),
                    Descriptor::new($params, $output),
                ),
                Box::new($method),
            )
            .unwrap();
    };
}

pub fn register(interface: &mut NativeInterface) {
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
    register_method!(
        interface,
        object_get_class,
        "java/lang/Object",
        "getClass",
        vec![],
        Type::Single(SingleType::Object("java/lang/Class".to_string()))
    );
    register_method!(
        interface,
        class_get_name,
        "java/lang/Class",
        "getName",
        vec![],
        Type::Single(SingleType::Object("java/lang/String".to_string()))
    );
}

fn string_get_internal(env: &mut NativeEnv) -> Primitive {
    let variables = env.variables().clone();

    let value = if let [this, ..] = &variables[..] {
        let this = env.reference(&this).unwrap();

        if this.fields.contains_key(&"<raw>".to_string()) {
            return this.fields.get(&"<raw>".to_string()).unwrap().clone();
        } else if let RustObject::String(value) = this.inner() {
            value
        } else {
            todo!()
        }
    } else {
        todo!()
    };

    let raw_array: Vec<Primitive> = Vec::from_iter(value.as_bytes())
        .iter()
        .map(|c| Primitive::Byte(**c))
        .collect();

    let array = Object::new_array(raw_array).unwrap();

    let index = Primitive::Reference(env.alloc(array).unwrap());

    if let [this, ..] = &variables[..] {
        let this = env.reference_mut(&this).unwrap();
        this.fields.insert("<raw>".to_string(), index.clone());
    }

    index
}

fn string_set_internal(env: &mut NativeEnv) -> Primitive {
    let variables = &env.variables().clone();

    if let [this, raw, ..] = &variables[..] {
        let raw = env.reference(raw).unwrap();

        let mut raw_array: Vec<u8> = Vec::new();
        if let RustObject::Array(raw) = raw.inner() {
            for value in raw.iter() {
                if let Primitive::Byte(r#char) = value {
                    raw_array.push(*r#char);
                } else {
                    break;
                }
            }
        } else {
            todo!()
        }

        let this = env.reference_mut(this).unwrap();
        *this.inner_mut() = RustObject::String(String::from_utf8(raw_array).unwrap());
    };

    Primitive::Void
}

fn object_get_class(env: &mut NativeEnv) -> Primitive {
    let variables = &env.variables().clone();

    if let [this] = &variables[..] {
        let object = env.reference(this).unwrap();
        Primitive::Reference(object.class().unwrap())
    } else {
        todo!()
    }
}

fn class_get_name(env: &mut NativeEnv) -> Primitive {
    let variables = &env.variables().clone();

    if let [this] = &variables[..] {
        if let Primitive::Reference(this) = this {
            let class = env.global().class(*this).unwrap();
            let class_name = class.metadata().this_class().clone();

            Primitive::Reference(env.new_string(class_name).unwrap())
        } else {
            todo!()
        }
    } else {
        todo!()
    }
}
