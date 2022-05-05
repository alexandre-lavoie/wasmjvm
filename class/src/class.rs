use crate::{
    AccessFlags, Attribute, AttributeInfo, Constant, ConstantInfo, Field, FieldInfo, Interface,
    InterfaceInfo, Method, MethodInfo, MethodRef, SourceStream, WithAccessFlags, WithAttributes,
    WithDescriptor, WithFields, WithInterfaces, WithMethods,
};

use std::slice::Iter;
use wasmjvm_common::{FromData, Parsable, Streamable, WasmJVMError};

#[derive(Debug)]
pub struct ClassFile {
    minor_version: u16,
    major_version: u16,
    constant_pool: Vec<ConstantInfo>,
    access_flags: AccessFlags,
    this_class: u16,
    super_class: u16,
    interfaces: Vec<InterfaceInfo>,
    fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
    attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub struct Class {
    constant_pool: Vec<Constant>,
    access_flags: AccessFlags,
    this_class: String,
    super_class: Option<String>,
    interfaces: Vec<Interface>,
    fields: Vec<Field>,
    methods: Vec<Method>,
    attributes: Vec<Attribute>,
}

impl Class {
    pub fn from_file<F: std::io::Read>(cursor: F) -> Result<Class, WasmJVMError> {
        let mut stream = SourceStream::from_file(cursor)?;
        Self::from_stream(&mut stream)
    }

    pub fn method_index(self: &Self, method_ref: &MethodRef) -> Result<usize, WasmJVMError> {
        for (index, method) in self.methods().unwrap().enumerate() {
            if &method_ref.name == method.name() && &method_ref.descriptor == method.descriptor() {
                return Ok(index);
            }
        }

        Err(WasmJVMError::NoSuchMethodError(format!("{:?}", method_ref)))
    }

    pub fn method(self: &Self, index: usize) -> &Method {
        &self.methods[index]
    }

    pub fn method_refs(self: &Self, name: &str) -> Result<Vec<MethodRef>, WasmJVMError> {
        let mut refs = Vec::new();

        for method in self.methods.iter() {
            if name == method.name() {
                refs.push(MethodRef::new(
                    self.this_class().to_string(),
                    method.name().to_string(),
                    method.descriptor().clone(),
                ));
            }
        }

        Ok(refs)
    }

    pub fn constant_pool(self: &Self) -> &Vec<Constant> {
        &self.constant_pool
    }

    pub fn constant(self: &Self, index: usize) -> &Constant {
        &self.constant_pool[index - 1]
    }

    pub fn access_flags(self: &Self) -> &AccessFlags {
        &self.access_flags
    }

    pub fn this_class(self: &Self) -> &str {
        &self.this_class
    }

    pub fn super_class(self: &Self) -> &Option<String> {
        &self.super_class
    }
}

impl ClassFile {
    pub fn minor_version(self: &Self) -> u16 {
        self.minor_version
    }

    pub fn major_version(self: &Self) -> u16 {
        self.major_version
    }

    pub fn constant(self: &Self, index: usize) -> Result<Constant, WasmJVMError> {
        self.constant_pool[index - 1].resolve(self)
    }

    pub fn resolve<T, K: ClassResolvable<T>>(self: &Self, target: &K) -> Result<T, WasmJVMError> {
        target.resolve(self)
    }

    pub fn resolve_vec<T, K: ClassResolvable<T>>(
        self: &Self,
        target: &Vec<K>,
    ) -> Result<Vec<T>, WasmJVMError> {
        let mut output: Vec<T> = Vec::with_capacity(target.capacity());

        for t in target.iter() {
            output.push(self.resolve(t)?);
        }

        Ok(output)
    }

    pub fn resolve_self(self: &Self) -> Result<Class, WasmJVMError> {
        let mut constant_pool = Vec::with_capacity(self.constant_pool.len());
        for i in 1..(self.constant_pool.len() + 1) {
            constant_pool.push(self.constant(i)?);
        }

        let access_flags = self.access_flags.clone();
        let this_class = self.constant(self.this_class as usize)?.to_string()?;
        let super_class = if self.super_class == 0 {
            None
        } else {
            Some(self.constant(self.super_class as usize)?.to_string()?)
        };
        let interfaces = self.resolve_vec(&self.interfaces)?;
        let fields = self.resolve_vec(&self.fields)?;
        let methods = self.resolve_vec(&self.methods)?;
        let attributes = self.resolve_vec(&self.attributes)?;

        Ok(Class {
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }
}

pub trait ClassResolvable<T> {
    fn resolve(self: &Self, class_file: &ClassFile) -> Result<T, WasmJVMError>;
}

impl Streamable<SourceStream, Class> for Class {
    fn from_stream(stream: &mut SourceStream) -> Result<Class, WasmJVMError> {
        let class_file: ClassFile = stream.parse()?;
        class_file.resolve_self()
    }
}

impl Streamable<SourceStream, ClassFile> for ClassFile {
    fn from_stream(stream: &mut SourceStream) -> Result<ClassFile, WasmJVMError> {
        let magic_number: u32 = stream.parse()?;
        if magic_number != 0xCAFEBABE {
            return Err(WasmJVMError::ClassFormatError(format!("Bad magic {}", magic_number)));
        }

        let minor_version = stream.parse()?;
        let major_version = stream.parse()?;

        let constant_pool_size: u16 = stream.parse()?;
        let mut constant_pool = Vec::with_capacity(constant_pool_size as usize);
        let mut ci = 0usize;
        while ci < (constant_pool_size as usize - 1usize) {
            let cp: ConstantInfo = stream.parse()?;

            match cp {
                ConstantInfo::Long(..) | ConstantInfo::Double(..) => {
                    constant_pool.push(cp);
                    constant_pool.push(ConstantInfo::Empty);
                    ci += 2;
                }
                _ => {
                    constant_pool.push(cp);
                    ci += 1;
                }
            }
        }

        let access_flags = stream.parse()?;

        let this_class = stream.parse()?;
        let super_class = stream.parse()?;

        let interface_count: u16 = stream.parse()?;
        let interfaces = stream.parse_vec(interface_count as usize)?;

        let field_count: u16 = stream.parse()?;
        let fields = stream.parse_vec(field_count as usize)?;

        let method_count: u16 = stream.parse()?;
        let methods = stream.parse_vec(method_count as usize)?;

        let attribute_count: u16 = stream.parse()?;
        let attributes = stream.parse_vec(attribute_count as usize)?;

        Ok(ClassFile {
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }
}

impl WithMethods for Class {
    fn methods(self: &Self) -> Option<Iter<Method>> {
        Some(self.methods.iter())
    }
}

impl WithFields for Class {
    fn fields(self: &Self) -> Option<Iter<Field>> {
        Some(self.fields.iter())
    }
}

impl WithAttributes for Class {
    fn attributes(self: &Self) -> Option<Iter<Attribute>> {
        Some(self.attributes.iter())
    }
}

impl WithInterfaces for Class {
    fn interfaces(self: &Self) -> Option<Iter<Interface>> {
        Some(self.interfaces.iter())
    }
}

impl WithAccessFlags for Class {
    fn access_flags(self: &Self) -> &AccessFlags {
        &self.access_flags
    }
}
