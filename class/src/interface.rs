use crate::{ClassFile, ClassResolvable, SourceStream};

use wasmjvm_common::{WasmJVMError, Streamable, Parsable};
use std::slice::Iter;

#[derive(Debug, Default)]
pub struct InterfaceInfo {
    name_index: u16,
}

#[derive(Debug, Clone)]
pub struct Interface {
    name: String,
}

impl Interface {
    fn name(self: &Self) -> &str {
        self.name.as_str()
    }
}

pub trait WithInterfaces {
    fn interfaces(self: &Self) -> Option<Iter<Interface>>;

    fn interface(self: &Self, name: &str) -> Result<&Interface, WasmJVMError> {
        if let Some(interfaces) = self.interfaces() {
            for interface in interfaces {
                if interface.name() == name {
                    return Ok(interface);
                }
            }
        }

        Err(WasmJVMError::ClassNotFoundException(format!("Interface {}", name)))
    }
}

impl ClassResolvable<Interface> for InterfaceInfo {
    fn resolve(self: &Self, class_file: &ClassFile) -> Result<Interface, WasmJVMError> {
        let name = class_file.constant(self.name_index as usize)?.to_string()?;

        Ok(Interface { name })
    }
}

impl Streamable<SourceStream, InterfaceInfo> for InterfaceInfo {
    fn from_stream(stream: &mut SourceStream) -> Result<InterfaceInfo, WasmJVMError> {
        let name_index = stream.parse()?;

        Ok(InterfaceInfo { name_index })
    }
}
