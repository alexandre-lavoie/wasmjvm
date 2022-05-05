use crate::{
    AccessFlags, Attribute, AttributeInfo, ClassFile, ClassResolvable, Descriptor, SourceStream,
    WithAccessFlags, WithAttributes, WithDescriptor,
};

use std::slice::Iter;
use wasmjvm_common::{Parsable, Streamable, WasmJVMError};

#[derive(Debug)]
pub struct MethodInfo {
    access_flags: AccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone)]
pub struct Method {
    access_flags: AccessFlags,
    name: String,
    descriptor: Descriptor,
    attributes: Vec<Attribute>,
}

pub trait WithMethods {
    fn methods(self: &Self) -> Option<Iter<Method>>;

    fn method(self: &Self, name: &str) -> Result<&Method, WasmJVMError> {
        if let Some(methods) = self.methods() {
            for method in methods {
                if method.name() == name {
                    return Ok(method);
                }
            }
        }

        Err(WasmJVMError::NoSuchMethodError(format!("{}", name)))
    }
}

impl Method {
    pub fn name(self: &Self) -> &str {
        self.name.as_str()
    }
}

impl WithAttributes for Method {
    fn attributes(self: &Self) -> Option<Iter<Attribute>> {
        Some(self.attributes.iter())
    }
}

impl WithAccessFlags for Method {
    fn access_flags(self: &Self) -> &AccessFlags {
        &self.access_flags
    }
}

impl WithDescriptor for Method {
    fn descriptor(self: &Self) -> &Descriptor {
        &self.descriptor
    }
}

impl ClassResolvable<Method> for MethodInfo {
    fn resolve(self: &Self, class_file: &ClassFile) -> Result<Method, WasmJVMError> {
        let access_flags = self.access_flags.clone();
        let name = class_file.constant(self.name_index as usize)?.to_string()?;
        let descriptor = class_file
            .constant(self.descriptor_index as usize)?
            .to_descriptor()?;
        let attributes = class_file.resolve_vec(&self.attributes)?;
        Ok(Method {
            access_flags,
            name,
            descriptor,
            attributes,
        })
    }
}

impl Streamable<SourceStream, MethodInfo> for MethodInfo {
    fn from_stream(stream: &mut SourceStream) -> Result<MethodInfo, WasmJVMError> {
        let access_flags = stream.parse()?;
        let name_index = stream.parse()?;
        let descriptor_index = stream.parse()?;

        let attribute_count: u16 = stream.parse()?;
        let attributes = stream.parse_vec(attribute_count as usize)?;

        Ok(MethodInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }
}
