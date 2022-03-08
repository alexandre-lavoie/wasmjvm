

#[derive(Debug)]
pub enum WasmJVMError {
    TODO,
    ClassNotFoundException(String),
    InstantiationException(String),
    NoSuchFieldException(String),
    NoSuchMethodException(String),
    ArithmeticException(String),
    ArrayStoreException(String),
    ClassCastException(String),
    EnumConstantNotPresentException(String),
    IllegalArgumentException(String),
    IllegalCallerException(String),
    IllegalStateException(String),
    IndexOutOfBoundException(String),
    LayerInstantiateException(String),
    NegativeArraySizeException(String),
    NullPointerException(String),
    SecurityException(String),
    TypeNotPresentException(String),
    UnsupportedOperationException(String),
    OutOfMemoryError(String),
    StackOverflowError(String),
    LinkageError(String),
    NoSuchFieldError(String),
    NoSuchMethodError(String),
    ClassFormatError(String)
}
