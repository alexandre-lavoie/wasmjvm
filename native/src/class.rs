use std::collections::HashMap;

use wasmjvm_class::{Class, WithFields};

use crate::Primitive;

pub const JAVA_OBJECT: &str = "java/lang/Object";
pub const JAVA_STRING: &str = "java/lang/String";
pub const JAVA_CLASS: &str = "java/lang/Class";
pub const JAVA_NATIVE: &str = "java/lang/Native";
pub const JAVA_LOADER: &str = "java/lang/Loader";
pub const JAVA_THREAD: &str = "java/lang/Thread";

#[derive(Debug)]
pub struct ClassInstance {
    metadata: Class,
    pub statics: HashMap<String, Primitive>
}

impl ClassInstance {
    pub fn new(metadata: Class) -> Self {
        let mut statics = HashMap::new();

        for field in metadata.static_field_names() {
            statics.insert(field, Primitive::Null);
        }

        Self {
            metadata,
            statics
        }
    }

    pub fn metadata(self: &Self) -> &Class {
        &self.metadata
    }
}
