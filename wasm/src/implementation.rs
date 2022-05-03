use crate::binding::JS;

use wasmjvm_class::{Descriptor, MethodRef, SingleType, Type};
use wasmjvm_native::{Primitive, NativeEnv, NativeInterface, register_method, RustObject};

pub fn register(interface: &mut NativeInterface) {
    register_method!(
        interface,
        file_bind,
        "java/io/FileInputStream",
        "nativeBind",
        vec![],
        Type::Single(SingleType::Void)
    );

    register_method!(
        interface,
        file_read,
        "java/io/FileInputStream",
        "nativeRead",
        vec![],
        Type::Single(SingleType::Int)
    );

    register_method!(
        interface,
        file_bind,
        "java/io/FileOutputStream",
        "nativeBind",
        vec![],
        Type::Single(SingleType::Void)
    );

    register_method!(
        interface,
        file_write,
        "java/io/FileOutputStream",
        "nativeWrite",
        vec![
            Type::Single(SingleType::Int)
        ],
        Type::Single(SingleType::Void)
    );
}

fn file_bind(env: &mut NativeEnv) -> Primitive {
    if let [this_ref, ..] = &env.variables()[..] {
        if let Primitive::Reference(this_index) = this_ref {
            let this = env.reference(&this_ref).unwrap();
            let path_ref = this.fields.get("path").unwrap();
            let path_object = env.reference(&path_ref).unwrap();
            if let RustObject::String(path) = path_object.inner() {
                JS::file_bind(*this_index, path.clone());

                return Primitive::Void;
            }
        }
    }

    unreachable!();
}

fn file_read(env: &mut NativeEnv) -> Primitive {
    if let [this_ref, ..] = &env.variables()[..] {
        if let Primitive::Reference(this_index) = this_ref {
            return Primitive::Int(JS::file_read(*this_index));
        }
    }

    unreachable!()
}

fn file_write(env: &mut NativeEnv) -> Primitive {
    if let [this_ref, value, ..] = &env.variables()[..] {
        if let Primitive::Reference(this_index) = this_ref {
                if let Primitive::Int(value) = value {
                    JS::file_write(*this_index, *value);

                    return Primitive::Void;
                }
        }
    }

    unreachable!()
}
