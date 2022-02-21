use crate::SourceStream;

use wasmjvm_common::{WasmJVMError, Streamable, Parsable};
use std::collections::HashSet;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AccessFlagType {
    Public = 0x0001,
    Private = 0x0002,
    Protected = 0x0004,
    Static = 0x0008,
    Final = 0x0010,
    Super = 0x0020,
    Volatile = 0x0040,
    Transient = 0x0080,
    Native = 0x0100,
    Interface = 0x0200,
    Abstract = 0x0400,
    Strict = 0x0800,
    Synthetic = 0x1000,
    Annotation = 0x2000,
    Enum = 0x4000,
}

#[derive(Default, Debug, Clone)]
pub struct AccessFlags {
    types: HashSet<AccessFlagType>,
}

pub trait WithAccessFlags {
    fn access_flags(self: &Self) -> &AccessFlags;
}

impl AccessFlags {
    pub fn has_type(self: &Self, flag_type: &AccessFlagType) -> bool {
        self.types.contains(flag_type)
    }
}

impl Streamable<SourceStream, AccessFlags> for AccessFlags {
    fn from_stream(stream: &mut SourceStream) -> Result<AccessFlags, WasmJVMError> {
        let mut types = HashSet::new();
        let flags: u16 = stream.parse()?;

        let flag_types = [
            AccessFlagType::Public,
            AccessFlagType::Private,
            AccessFlagType::Protected,
            AccessFlagType::Static,
            AccessFlagType::Final,
            AccessFlagType::Super,
            AccessFlagType::Volatile,
            AccessFlagType::Transient,
            AccessFlagType::Native,
            AccessFlagType::Interface,
            AccessFlagType::Abstract,
            AccessFlagType::Strict,
            AccessFlagType::Synthetic,
            AccessFlagType::Annotation,
            AccessFlagType::Enum,
        ];

        for flag_type in flag_types {
            if flag_type as u16 & flags != 0 {
                types.insert(flag_type);
            }
        }

        Ok(AccessFlags { types })
    }
}
