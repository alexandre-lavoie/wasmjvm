use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crate::{Global, Object, Primitive};
use wasmjvm_class::MethodRef;
use wasmjvm_common::WasmJVMError;

pub type NativeFn = Box<dyn Fn(&mut NativeEnv) -> Primitive>;

#[derive(Clone)]
pub struct NativeMethod {
    raw: Arc<NativeFn>,
}

impl NativeMethod {
    fn new(r#fn: NativeFn) -> Self {
        Self {
            raw: Arc::new(r#fn),
        }
    }

    pub fn invoke(self: &Self, env: &mut NativeEnv) -> Primitive {
        (self.raw)(env)
    }
}

#[derive(Default)]
pub struct NativeInterface {
    methods: HashMap<MethodRef, NativeMethod>,
}

impl Debug for NativeInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeInterface").finish()
    }
}

impl NativeInterface {
    pub fn new() -> Self {
        Self {
            methods: HashMap::new(),
        }
    }

    pub fn register(
        self: &mut Self,
        method_ref: MethodRef,
        r#fn: NativeFn,
    ) -> Result<(), WasmJVMError> {
        if self.methods.contains_key(&method_ref) {
            return Err(WasmJVMError::TODO(24));
        }

        self.methods.insert(method_ref, NativeMethod::new(r#fn));

        Ok(())
    }

    pub fn method(self: &Self, method_ref: &MethodRef) -> Result<NativeMethod, WasmJVMError> {
        if let Some(method) = self.methods.get(method_ref) {
            Ok(method.clone())
        } else {
            Err(WasmJVMError::TODO(25))
        }
    }
}

#[derive(Debug)]
pub struct NativeEnv {
    global: Global,
    variables: Vec<Primitive>,
}

impl NativeEnv {
    pub fn new(global: Global, variables: Vec<Primitive>) -> Self {
        Self { global, variables }
    }

    pub fn global(self: &Self) -> &Global {
        &self.global
    }

    pub fn variables(self: &Self) -> &Vec<Primitive> {
        &self.variables
    }

    pub fn variables_mut(self: &mut Self) -> &mut Vec<Primitive> {
        &mut self.variables
    }

    pub fn new_string(self: &mut Self, string: String) -> Result<usize, WasmJVMError> {
        self.global.new_java_string(string)
    }

    pub fn alloc(self: &mut Self, object: Object) -> Result<usize, WasmJVMError> {
        self.global.new_object(object)
    }

    pub fn reference(self: &Self, reference: &Primitive) -> Result<&Object, WasmJVMError> {
        self.global.reference_p(reference)
    }

    pub fn reference_mut(
        self: &mut Self,
        reference: &Primitive,
    ) -> Result<&mut Object, WasmJVMError> {
        self.global.reference_p_mut(reference)
    }
}
