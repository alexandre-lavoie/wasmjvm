use std::collections::HashMap;

use wasmjvm_class::{Class, WithFields};

use crate::Primitive;

pub static JAVA_OBJECT: &str = "java/lang/Object";
pub static JAVA_STRING: &str = "java/lang/String";
pub static JAVA_CLASS: &str = "java/lang/Class";
pub static JAVA_NATIVE: &str = "java/lang/Native";
pub static JAVA_LOADER: &str = "java/lang/Loader";
pub static JAVA_THREAD: &str = "java/lang/Thread";

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
