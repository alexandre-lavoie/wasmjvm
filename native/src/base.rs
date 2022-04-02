use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use wasmjvm_class::{Descriptor, FieldRef, MethodRef, WithFields};
use wasmjvm_common::WasmJVMError;

use crate::{
    ClassInstance, Loader, NativeEnv, NativeFn, NativeInterface, Object, Primitive, RustObject,
    Thread, ThreadResult, JAVA_STRING,
};

pub type RegisterFn = Box<dyn Fn(&mut NativeInterface)>;

static mut HEAP: Option<Vec<Option<Object>>> = None;

#[derive(Debug, Default, Clone)]
pub struct Heap {
    index: Arc<Mutex<usize>>,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalData {
    main_class_index: Option<usize>,
    native_index: Option<usize>,
    loader_index: Option<usize>,
    classes: HashMap<String, usize>,
    threads: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct Global {
    heap: Heap,
    data: Arc<Mutex<GlobalData>>,
}

impl Heap {
    pub fn get(self: &Self, index: usize) -> Result<&Object, WasmJVMError> {
        unsafe {
            if let Some(heap) = &HEAP {
                if let Some(value) = heap.get(index) {
                    if let Some(value) = value.as_ref() {
                        return Ok(value);
                    }
                }
            }
        }

        Err(WasmJVMError::TODO)
    }

    fn get_mut(self: &Self, index: usize) -> Result<&mut Object, WasmJVMError> {
        unsafe {
            if let Some(heap) = &mut HEAP {
                if let Some(value) = heap.get_mut(index) {
                    if let Some(value) = value.as_mut() {
                        return Ok(value);
                    }
                }
            }
        }

        Err(WasmJVMError::TODO)
    }

    pub fn index(self: &Self) -> usize {
        unsafe {
            if let Some(heap) = &HEAP {
                if let Ok(index) = self.index.lock() {
                    if *index >= heap.len() {
                        panic!("Out of heap")
                    }

                    *index
                } else {
                    unreachable!();
                }
            } else {
                0
            }
        }
    }

    pub fn alloc(self: &mut Self, object: Object) -> Result<usize, WasmJVMError> {
        unsafe {
            if HEAP.is_none() {
                let mut heap: Vec<Option<Object>> = Vec::with_capacity(1024);

                for _ in 0..heap.capacity() {
                    heap.push(None);
                }

                HEAP = Some(heap)
            }
        }

        let index = if let Ok(mut index) = self.index.lock() {
            unsafe {
                if let Some(heap) = &mut HEAP {
                    heap[*index] = Some(object);
                }
            }

            *index += 1;

            *index - 1
        } else {
            unreachable!();
        };

        Ok(index)
    }
}

impl Global {
    pub fn new() -> Global {
        Global {
            ..Default::default()
        }
    }

    pub fn index(self: &Self) -> usize {
        self.heap.index()
    }

    pub fn native_index(self: &Self) -> Result<usize, WasmJVMError> {
        if let Ok(data) = self.data.lock() {
            if let Some(native_index) = data.native_index.clone() {
                Ok(native_index)
            } else {
                Err(WasmJVMError::TODO)
            }
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn native(self: &Self) -> Result<&NativeInterface, WasmJVMError> {
        let object = self.reference(self.native_index()?)?;

        if let RustObject::Native(native) = object.inner() {
            Ok(native)
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn native_mut(self: &mut Self) -> Result<&mut NativeInterface, WasmJVMError> {
        let object = self.reference_mut(self.native_index()?)?;

        if let RustObject::Native(native) = object.inner_mut() {
            Ok(native)
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn register_native(self: &mut Self, r#fn: RegisterFn) -> Result<(), WasmJVMError> {
        r#fn(self.native_mut()?);

        Ok(())
    }

    pub fn class(self: &Self, index: usize) -> Result<&ClassInstance, WasmJVMError> {
        let object = self.reference(index)?;

        if let RustObject::Class(class) = object.inner() {
            Ok(class)
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn class_mut(self: &mut Self, index: usize) -> Result<&mut ClassInstance, WasmJVMError> {
        let object = self.reference_mut(index)?;

        if let RustObject::Class(class) = object.inner_mut() {
            Ok(class)
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    fn loader_index(self: &Self) -> Result<usize, WasmJVMError> {
        if let Ok(data) = self.data.lock() {
            if let Some(loader_index) = data.loader_index.clone() {
                Ok(loader_index)
            } else {
                Err(WasmJVMError::TODO)
            }
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn loader(self: &Self) -> Result<&Loader, WasmJVMError> {
        let object = self.reference(self.loader_index()?)?;

        if let RustObject::Loader(loader) = object.inner() {
            Ok(loader)
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn loader_mut(self: &mut Self) -> Result<&mut Loader, WasmJVMError> {
        let object = self.reference_mut(self.loader_index()?)?;

        if let RustObject::Loader(loader) = object.inner_mut() {
            Ok(loader)
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn thread_tick(self: &mut Self, thread_ref: usize) -> Result<ThreadResult, WasmJVMError> {
        let object_mut = self.reference_mut(thread_ref)?;

        if let RustObject::Thread(thread) = object_mut.inner_mut() {
            thread.tick()
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn threads(self: &mut Self) -> Vec<usize> {
        if let Ok(data) = self.data.lock() {
            data.threads.clone()
        } else {
            unreachable!()
        }
    }

    pub fn native_register(
        self: &mut Self,
        method_ref: MethodRef,
        r#fn: NativeFn,
    ) -> Result<(), WasmJVMError> {
        let native_index = if let Ok(data) = self.data.lock() {
            if let Some(native_index) = data.native_index {
                native_index
            } else {
                return Err(WasmJVMError::TODO);
            }
        } else {
            return Err(WasmJVMError::TODO);
        };

        let object_mut = self.heap.get_mut(native_index)?;
        if let RustObject::Native(native) = object_mut.inner_mut() {
            native.register(method_ref, r#fn)?;
        } else {
            return Err(WasmJVMError::TODO);
        }

        Ok(())
    }

    pub fn native_invoke(
        self: &mut Self,
        method_ref: &MethodRef,
        variables: Vec<Primitive>,
    ) -> Result<Primitive, WasmJVMError> {
        let method = self.native_mut()?.method(method_ref)?;
        let mut env = NativeEnv::new(self.clone(), variables);
        Ok(method.invoke(&mut env))
    }

    pub fn reference_p(self: &Self, reference: &Primitive) -> Result<&Object, WasmJVMError> {
        if let Primitive::Reference(index) = reference {
            self.reference(*index)
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn reference(self: &Self, reference: usize) -> Result<&Object, WasmJVMError> {
        self.heap.get(reference)
    }

    pub fn reference_p_mut(
        self: &mut Self,
        reference: &Primitive,
    ) -> Result<&mut Object, WasmJVMError> {
        if let Primitive::Reference(index) = reference {
            self.reference_mut(*index)
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn reference_mut(self: &mut Self, reference: usize) -> Result<&mut Object, WasmJVMError> {
        self.heap.get_mut(reference)
    }

    pub fn thread_mut(self: &mut Self, index: usize) -> Result<&mut Thread, WasmJVMError> {
        let object = self.reference_mut(index)?;

        if let RustObject::Thread(thread) = object.inner_mut() {
            Ok(thread)
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn array_set(
        self: &mut Self,
        reference: Primitive,
        index: Primitive,
        value: Primitive,
    ) -> Result<(), WasmJVMError> {
        let object = self.reference_p_mut(&reference)?;

        if let (RustObject::Array(array), Primitive::Int(index)) = (object.inner_mut(), index) {
            array[index as usize] = value;

            return Ok(());
        }

        Err(WasmJVMError::TODO)
    }

    pub fn static_field(self: &Self, field_ref: &FieldRef) -> Result<Primitive, WasmJVMError> {
        let class_ref = self.class_index(&field_ref.class)?;

        if let Some(field) = self.class(class_ref)?.statics.get(&field_ref.name) {
            Ok(field.clone())
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn static_field_set(
        self: &mut Self,
        field_ref: &FieldRef,
        value: Primitive,
    ) -> Result<(), WasmJVMError> {
        let class_index = self.class_index(&field_ref.class)?;
        let class = self.class_mut(class_index)?;

        class.statics.insert(field_ref.name.clone(), value);

        Ok(())
    }

    pub fn field_set(
        self: &mut Self,
        this_ref: Primitive,
        field_ref: &FieldRef,
        value: Primitive,
    ) -> Result<(), WasmJVMError> {
        let object_mut = self.reference_p_mut(&this_ref)?;

        object_mut.fields.insert(field_ref.name.clone(), value);

        Ok(())
    }

    pub fn resolve_fields(self: &Self, this_ref: usize) -> Result<Vec<String>, WasmJVMError> {
        let object = self.reference(this_ref)?;

        let mut fields = Vec::new();
        let mut class_index = object.class();
        loop {
            if let Some(index) = class_index {
                let class = self.class(index)?;

                fields.append(&mut class.metadata().field_names());

                if let Some(super_name) = class.metadata().super_class() {
                    class_index = Some(self.class_index(super_name)?);
                } else {
                    class_index = None;
                }
            } else {
                break;
            }
        }

        Ok(fields)
    }

    pub fn main_class_set(self: &Self, class_name: &String) -> Result<(), WasmJVMError> {
        let class_ref = self.class_index(class_name)?;

        if let Ok(mut data) = self.data.lock() {
            data.main_class_index = Some(class_ref);

            return Ok(());
        }

        Err(WasmJVMError::ClassNotFoundException(format!(
            "Could not find main class {}",
            class_name
        )))
    }

    pub fn main_class_index(self: &Self) -> Result<usize, WasmJVMError> {
        if let Ok(data) = self.data.lock() {
            if let Some(index) = data.main_class_index {
                return Ok(index);
            }
        }

        Err(WasmJVMError::ClassNotFoundException(format!(
            "No main class set"
        )))
    }

    pub fn class_index(self: &Self, name: &String) -> Result<usize, WasmJVMError> {
        if let Ok(data) = self.data.lock() {
            if let Some(index) = data.classes.get(name) {
                return Ok(*index);
            }
        }

        Err(WasmJVMError::ClassNotFoundException(format!(
            "Could not find class {}",
            name
        )))
    }

    pub fn default_init(self: &mut Self, index: usize) -> Result<(), WasmJVMError> {
        let loader_index = self.loader_index()?;

        let object = self.heap.get_mut(loader_index)?;

        if let Some(class) = object.class() {
            let loader = if let RustObject::Loader(loader) = object.inner_mut() {
                loader
            } else {
                return Err(WasmJVMError::TODO);
            };

            loader.default_init(class, index)?;

            Ok(())
        } else {
            Err(WasmJVMError::TODO)
        }
    }

    pub fn method(
        self: &Self,
        method_ref: &MethodRef,
    ) -> Result<(usize, usize, Descriptor), WasmJVMError> {
        let class_index = self.class_index(&method_ref.class)?;
        let class = self.class(class_index)?;

        let method_index = class.metadata().method_index(&method_ref)?;
        let descriptor = method_ref.descriptor.clone();

        Ok((class_index, method_index, descriptor))
    }

    pub fn new_object(self: &mut Self, object: Object) -> Result<usize, WasmJVMError> {
        if let Ok(mut data) = self.data.lock() {
            let index = self.heap.index();

            match object.inner() {
                RustObject::Class(class) => {
                    // TODO: Check if duplicate insert.
                    data.classes
                        .insert(class.metadata().this_class().clone(), index);
                }
                RustObject::Loader(loader) => {
                    if data.loader_index.is_some() {
                        return Err(WasmJVMError::TODO);
                    }

                    data.loader_index = Some(index)
                }
                RustObject::Native(native) => {
                    if data.native_index.is_some() {
                        return Err(WasmJVMError::TODO);
                    }

                    data.native_index = Some(index)
                }
                RustObject::Thread(thread) => data.threads.push(index),
                _ => {}
            }

            self.heap.alloc(object)
        } else {
            unreachable!()
        }
    }

    pub fn new_instance(self: &mut Self, class: &String) -> Result<usize, WasmJVMError> {
        self.new_rust_instance(class, RustObject::Null)
    }

    pub fn new_rust_instance(
        self: &mut Self,
        class: &String,
        inner: RustObject,
    ) -> Result<usize, WasmJVMError> {
        let class_index = self.class_index(class)?;
        let class = self.class_mut(class_index)?;
        let fields = self.resolve_fields(class_index)?;

        let object = Object::new(class_index, fields, inner)?;

        self.new_object(object)
    }

    pub fn new_java_string(self: &mut Self, string: String) -> Result<usize, WasmJVMError> {
        let index = self.new_rust_instance(&JAVA_STRING.to_string(), RustObject::String(string))?;

        self.default_init(index)?;

        Ok(index)
    }
}
