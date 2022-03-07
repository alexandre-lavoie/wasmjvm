use std::{collections::HashMap, fmt::Debug};

use wasmjvm_class::{MethodRef, Object, Primitive, Descriptor};
use wasmjvm_common::WasmJVMError;

use crate::Heap;

pub type NativeFn = Box<dyn Fn(&mut NativeEnv) -> Primitive>;

#[derive(Default)]
pub struct NativeInterface {
    methods: HashMap<MethodRef, NativeFn>,
}

impl Debug for NativeInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeInterface").finish()
    }
}

impl NativeInterface {
    pub fn register(self: &mut Self, method_ref: MethodRef, r#fn: NativeFn) {
        self.methods.insert(method_ref, r#fn);
    }

    pub fn invoke(
        self: &mut Self,
        method_ref: &MethodRef,
        env: &mut NativeEnv,
    ) -> Result<Primitive, WasmJVMError> {
        if let Some(method) = self.methods.get(method_ref) {
            Ok(method(env))
        } else {
            Err(WasmJVMError::RuntimeError)
        }
    }
}

#[derive(Debug)]
pub struct NativeEnv<'a> {
    heap: &'a mut Heap,
    variables: Vec<Primitive>,
    instances: Vec<(usize, MethodRef)>
}

impl<'a> NativeEnv<'a> {
    pub fn new(heap: &'a mut Heap, variables: Vec<Primitive>) -> Self {
        Self { heap, variables, instances: Vec::new() }
    }

    pub fn variables(self: &Self) -> &Vec<Primitive> {
        &self.variables
    }

    pub fn instances(self: &Self) -> &Vec<(usize, MethodRef)> {
        &self.instances
    }

    pub fn new_string(self: &mut Self, string: String) -> Result<usize, WasmJVMError> {
        let index = self.heap.new_string(string)?;

        let init_ref = MethodRef::string_init();

        self.instances.push((index, init_ref));
        Ok(index)
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
}
