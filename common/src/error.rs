#[derive(Debug)]
pub enum WasmJVMError {
    FileNotFound,
    OutOfBound,
    MethodNotFound,
    ClassNotFound,
    InterfaceNotFound,
    FieldNotFound,
    AttributeNotFound,
    FieldInvalid,
    DescriptorInvalid,
    StringInvalid,
    TypeInvalid,
    ConstantInvalid,
    NameDescriptorInvalid,
    BadMagic,
    OpcodeInvalid,
    ClassInvalid,
    MethodInvalid
}
