use std::collections::HashMap;

use wasmjvm_class::{Class, WithFields};
use wasmjvm_common::WasmJVMError;

use crate::{
    ClassInstance, Global, Object, RustObject, Thread, JAVA_CLASS, JAVA_LOADER, JAVA_OBJECT,
    JAVA_THREAD,
};

#[derive(Debug)]
pub struct Loader {
    global: Global,
    clinit_thread: usize,
    init_thread: usize,
}

impl Loader {
    pub fn threads(self: &Self) -> (usize, usize) {
        (self.clinit_thread, self.init_thread)
    }

    pub fn new(global: Global) -> Loader {
        Self {
            global,
            clinit_thread: 0,
            init_thread: 0,
        }
    }

    pub fn clinit(self: &mut Self, class: usize) -> Result<(), WasmJVMError> {
        self.global.thread_lock(2)?;

        let thread = self.global.thread_mut(self.clinit_thread)?;
        thread.new_clinit_frame(class)?;

        Ok(())
    }

    pub fn default_init(self: &mut Self, class: usize, this: usize) -> Result<(), WasmJVMError> {
        self.global.thread_lock(1)?;

        let thread = self.global.thread_mut(self.init_thread)?;
        thread.new_default_init_frame(class, this)?;

        Ok(())
    }

    pub fn load_class(self: &mut Self, metadata: Class) -> Result<usize, WasmJVMError> {
        let class_index = self.global.class_index(&JAVA_CLASS.to_string())?;
        // TODO: Look at Jar instead?
        let is_main = metadata.method_refs(&"main".to_string())?.len() > 0;
        let class_name = metadata.this_class().clone();

        let inner = RustObject::Class(ClassInstance::new(metadata));
        let class = Object::new(class_index, self.global.resolve_fields(class_index)?, inner)?;

        let object_index = self.global.new_object(class)?;

        self.clinit(object_index)?;
        self.default_init(class_index, object_index)?;

        if is_main {
            self.global.set_main_class(&class_name)?;
        }

        Ok(object_index)
    }

    pub fn load_class_file<F: std::io::Read>(
        self: &mut Self,
        reader: F,
    ) -> Result<usize, WasmJVMError> {
        self.load_class(Class::from_file(reader)?)
    }

    pub fn load_jar<F: std::io::Read + std::io::Seek>(
        self: &mut Self,
        reader: F,
    ) -> Result<(), WasmJVMError> {
        let mut jar = zip::ZipArchive::new(reader).unwrap();

        for i in 0..jar.len() {
            let entry = jar.by_index(i);

            if let Ok(entry) = entry {
                if entry.is_file() {
                    if entry.name().ends_with(".class") {
                        self.load_class_file(entry)?;
                    }
                }
            } else {
                return Err(WasmJVMError::TODO(41));
            }
        }

        Ok(())
    }

    fn pop_boot_class(self: &mut Self, classes: &mut HashMap<String, Class>, class: &str) -> Result<Class, WasmJVMError> {
        if let Some(class) = classes.remove(&class.to_string()) {
            Ok(class)
        } else {
            Err(WasmJVMError::ClassNotFoundException(format!("Could not load boot class {}", class)))
        }
    }

    fn load_boot_classes(self: &mut Self, mut classes: HashMap<String, Class>) -> Result<(), WasmJVMError> {
        let object_class = self.pop_boot_class(&mut classes, JAVA_OBJECT)?;
        let object_fields = object_class.field_names();

        let mut clinits = Vec::new();
        let mut inits = Vec::new();

        let object = Object::new(
            self.global.index()?,
            object_fields.clone(),
            RustObject::Class(ClassInstance::new(object_class)),
        )?;
        let object_index = self.global.new_object(object)?;
        clinits.push(object_index);
        inits.push((object_index, object_index));

        let class_class = self.pop_boot_class(&mut classes, JAVA_CLASS)?;
        let mut class_fields = class_class.field_names();
        class_fields.append(&mut object_fields.clone());

        let class = Object::new(
            object_index.clone(),
            object_fields,
            RustObject::Class(ClassInstance::new(class_class)),
        )?;
        let class_index = self.global.new_object(class)?;
        clinits.push(class_index);
        inits.push((object_index, class_index));

        let loader_class = self.pop_boot_class(&mut classes, JAVA_LOADER)?;
        let loader = Object::new(
            class_index.clone(),
            class_fields.clone(),
            RustObject::Class(ClassInstance::new(loader_class)),
        )?;
        let loader_index = self.global.new_object(loader)?;
        clinits.push(loader_index);
        inits.push((class_index, loader_index));

        let thread_class = self.pop_boot_class(&mut classes, JAVA_THREAD)?;
        let thread_class = Object::new(
            class_index.clone(),
            class_fields.clone(),
            RustObject::Class(ClassInstance::new(thread_class)),
        )?;
        let thread_class_index = self.global.new_object(thread_class)?;
        clinits.push(thread_class_index);
        inits.push((class_index, thread_class_index));

        inits.push((thread_class_index, inits.len()));
        inits.push((thread_class_index, inits.len() + 1));

        let thread_fields = self.global.resolve_fields(thread_class_index)?;

        let mut clinit_thread = Thread::new(self.global.clone(), 2);
        for class in clinits.iter().rev() {
            clinit_thread.new_clinit_frame(*class)?;
        }
        let clinit_thread = Object::new(
            thread_class_index,
            thread_fields.clone(),
            RustObject::Thread(clinit_thread),
        )?;
        self.clinit_thread = self.global.new_object(clinit_thread)?;

        let mut init_thread = Thread::new(self.global.clone(), 1);
        for (class, this) in inits.iter().rev() {
            init_thread.new_default_init_frame(*class, *this)?;
        }
        let init_thread = Object::new(
            thread_class_index,
            thread_fields,
            RustObject::Thread(init_thread),
        )?;
        self.init_thread = self.global.new_object(init_thread)?;

        Ok(())
    }

    pub fn load_boot_jar<F: std::io::Read + std::io::Seek>(
        self: &mut Self,
        reader: F,
    ) -> Result<(), WasmJVMError> {
        let mut jar = zip::ZipArchive::new(reader).unwrap();

        let mut boot_classes: HashMap<String, Class> = HashMap::new();
        let mut classes: Vec<Class> = Vec::new();
        for i in 0..jar.len() {
            let entry = jar.by_index(i);

            if let Ok(entry) = entry {
                if entry.is_file() {
                    if entry.name().ends_with(".class") {
                        let class = Class::from_file(entry)?;

                        match class.this_class().as_str() {
                            JAVA_OBJECT | JAVA_CLASS | JAVA_LOADER | JAVA_THREAD => {
                                boot_classes.insert(class.this_class().clone(), class);
                            }
                            _ => classes.push(class),
                        }
                    }
                }
            } else {
                return Err(WasmJVMError::LinkageError(format!("{:?}", entry.err().unwrap())));
            }
        }

        self.load_boot_classes(boot_classes)?;

        for class in classes {
            self.load_class(class)?;
        }

        Ok(())
    }
}
