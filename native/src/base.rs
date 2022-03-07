use std::collections::HashMap;

use wasmjvm_class::{ClassInstance, Constant, Descriptor, MethodRef, Object, Primitive, Class, FieldRef};
use wasmjvm_common::WasmJVMError;

use crate::{NativeEnv, NativeFn, NativeInterface};

#[derive(Debug, Default)]
pub struct Heap {
    main_class: Option<usize>,
    classes: HashMap<String, usize>,
    objects: Vec<Object>,
}

#[derive(Debug, Default)]
pub struct Global {
    heap: Heap,
    pub native: NativeInterface,
}

impl Heap {
    pub fn get(self: &Self, index: usize) -> &Object {
        let object = &self.objects[index];

        object
    }

    pub fn get_mut(self: &mut Self, index: usize) -> &mut Object {
        let object = &mut self.objects[index];

        object
    }

    pub fn main_class_name(self: &Self) -> Result<&String, WasmJVMError> {
        let class_index = self.main_class_index()?;
        let class = self.class(class_index)?;
        Ok(class.metadata.this_class())
    }

    pub fn main_class_index(self: &Self) -> Result<usize, WasmJVMError> {
        if let Some(index) = self.main_class {
            Ok(index)
        } else {
            Err(WasmJVMError::ClassNotFound)
        }
    }

    pub fn class_index(self: &Self, name: &String) -> usize {
        if let Some(index) = self.classes.get(name) {
            *index
        } else {
            panic!("Unable to load class {}", name)
        }
    }

    pub fn class(self: &Self, index: usize) -> Result<&ClassInstance, WasmJVMError> {
        let class = &self.objects[index];
        match class {
            Object::Class(class) => Ok(class),
            _ => Err(WasmJVMError::ClassNotFound),
        }
    }

    pub fn class_mut(self: &mut Self, index: usize) -> Result<&mut ClassInstance, WasmJVMError> {
        let class = &mut self.objects[index];
        match class {
            Object::Class(class) => Ok(class),
            _ => Err(WasmJVMError::ClassNotFound),
        }
    }

    pub fn main_class(self: &Self) -> Result<&ClassInstance, WasmJVMError> {
        let main_class_index = self.main_class_index()?;
        self.class(main_class_index)
    }

    pub fn alloc(self: &mut Self, object: Object) -> Result<usize, WasmJVMError> {
        let index = self.objects.len();

        if let Object::Class(class) = &object {
            if self.main_class == None {
                self.main_class = Some(index);
            }

            // TODO: Check if duplicate insert.
            self.classes
                .insert(class.metadata.this_class().clone(), index);
        }

        self.objects.push(object);

        Ok(index)
    }

    pub fn reference(self: &Self, reference: &Primitive) -> Result<&Object, WasmJVMError> {
        match reference {
            Primitive::Reference(index) => Ok(self.get(*index)),
            _ => unreachable!("{:?}", reference),
        }
    }

    pub fn reference_mut(
        self: &mut Self,
        reference: &Primitive,
    ) -> Result<&mut Object, WasmJVMError> {
        match reference {
            Primitive::Reference(index) => Ok(self.get_mut(*index)),
            _ => panic!("Expected reference but got {:?}", reference),
        }
    }

    pub fn new_instance(self: &mut Self, class: &String) -> Result<usize, WasmJVMError> {
        let class_index = self.class_index(class);
        let metadata = &self.class(class_index)?.metadata;

        let object = Object::new_instance(class_index, metadata)?;

        self.alloc(object)
    }

    pub fn new_string(self: &mut Self, string: String) -> Result<usize, WasmJVMError> {
        let string_ref = self.new_instance(&"java/lang/String".to_string())?;
        let object = self.reference_mut(&Primitive::Reference(string_ref))?;

        if let Object::Instance(instance) = object {
            instance.fields.insert("<raw>".to_string(), Primitive::String(string));
        }

        Ok(string_ref)
    }
}

impl Global {
    pub fn main_class(self: &Self) -> Result<&ClassInstance, WasmJVMError> {
        self.heap.main_class()
    }

    pub fn register(self: &mut Self, method_ref: MethodRef, r#fn: NativeFn) {
        self.native.register(method_ref, r#fn)
    }

    pub fn invoke(
        self: &mut Self,
        method_ref: &MethodRef,
        variables: Vec<Primitive>,
    ) -> Result<(NativeEnv, Primitive), WasmJVMError> {
        let mut env = NativeEnv::new(&mut self.heap, variables);
        let result = self.native.invoke(method_ref, &mut env)?;
        Ok((env, result))
    }

    pub fn alloc(self: &mut Self, object: Object) -> Result<usize, WasmJVMError> {
        self.heap.alloc(object)
    }

    pub fn reference(self: &Self, reference: &Primitive) -> Result<&Object, WasmJVMError> {
        self.heap.reference(reference)
    }

    pub fn reference_mut(
        self: &mut Self,
        reference: &Primitive,
    ) -> Result<&mut Object, WasmJVMError> {
        self.heap.reference_mut(reference)
    }

    pub fn static_field(self: &Self, field_ref: &FieldRef) -> Result<&Primitive, WasmJVMError> {
        let class = self.class_index(&field_ref.class);
        let class = self.heap.class(class)?;

        Ok(class.statics.get(&field_ref.name).unwrap())
    }

    pub fn static_field_mut(
        self: &mut Self,
        field_ref: &FieldRef,
    ) -> Result<&mut Primitive, WasmJVMError> {
        let class = self.class_index(&field_ref.class);
        let class = self.heap.class_mut(class)?;

        Ok(class.statics.get_mut(&field_ref.name).unwrap())
    }

    pub fn main_class_index(self: &Self) -> Result<usize, WasmJVMError> {
        self.heap.main_class_index()
    }

    pub fn class(self: &Self, index: usize) -> Result<&ClassInstance, WasmJVMError> {
        self.heap.class(index)
    }

    pub fn class_index(self: &Self, name: &String) -> usize {
        self.heap.class_index(name)
    }

    pub fn method(
        self: &Self,
        method_ref: &MethodRef,
    ) -> Result<(usize, usize, Descriptor), WasmJVMError> {
        let class_index = self.class_index(&method_ref.class);
        let class_object = self.class(class_index)?;
        let method_index = class_object.metadata.method_index(&method_ref)?;
        let descriptor = method_ref.descriptor.clone();

        Ok((class_index, method_index, descriptor))
    }

    pub fn new_instance(self: &mut Self, class: &String) -> Result<usize, WasmJVMError> {
        self.heap.new_instance(class)
    }

    pub fn new_string(self: &mut Self, string: String) -> Result<usize, WasmJVMError> {
        self.heap.new_string(string)
    }
}
