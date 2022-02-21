use crate::{
    AccessFlags, Attribute, AttributeInfo, Constant, ConstantInfo, Field, FieldInfo, Interface,
    InterfaceInfo, Method, MethodInfo, SourceStream, WithAccessFlags, WithAttributes, WithFields,
    WithInterfaces, WithMethods,
};

use std::slice::Iter;
use wasmjvm_common::{Parsable, Streamable, WasmJVMError, FromData};

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
    super_class: String,
    interfaces: Vec<Interface>,
    fields: Vec<Field>,
    methods: Vec<Method>,
    attributes: Vec<Attribute>,
}

impl Class {
    pub fn from_string(path: &String) -> Result<Class, WasmJVMError> {
        let mut stream = SourceStream::from_file(path)?;
        Self::from_stream(&mut stream)
    }

    pub fn constant(self: &Self, index: usize) -> &Constant {
        &self.constant_pool[index]
    }

    pub fn access_flags(self: &Self) -> &AccessFlags {
        &self.access_flags
    }

    pub fn this_class(self: &Self) -> &String {
        &self.this_class
    }

    pub fn super_class(self: &Self) -> &String {
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
        let super_class = self.constant(self.super_class as usize)?.to_string()?;
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
            return Err(WasmJVMError::BadMagic);
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
