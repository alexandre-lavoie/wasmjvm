use wasmjvm_class::{Class, WithFields};
use wasmjvm_common::WasmJVMError;

use crate::{
    ClassInstance, Global, Object, RustObject, Thread, JAVA_CLASS, JAVA_LOADER, JAVA_OBJECT,
    JAVA_THREAD,
};

trait Resource {
    fn load_class(self: &mut Self, name: &str) -> Result<Class, WasmJVMError>;
}

pub struct Jar<F: std::io::Read + std::io::Seek> {
    zip_file: zip::ZipArchive<F>
}

impl<F> Jar<F> where F: std::io::Read + std::io::Seek {
    pub fn new(reader: F) -> Self {
        Self {
            zip_file: zip::ZipArchive::new(reader).unwrap()
        }
    } 
}

impl<F> Resource for Jar<F> where F: std::io::Read + std::io::Seek {
    fn load_class(self: &mut Self, name: &str) -> Result<Class, WasmJVMError> {
        if let Ok(file) = self.zip_file.by_name(format!("{}.class", name).as_str()) {
            Class::from_file(file)
        } else {
            Err(WasmJVMError::ClassNotFoundException(format!("{}", name)))
        }
    }
}

pub struct Loader {
    global: Global,
    clinit_thread: usize,
    init_thread: usize,
    resources: Vec<Box<dyn Resource>>
}

impl std::fmt::Debug for Loader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Loader").field("global", &self.global).field("clinit_thread", &self.clinit_thread).field("init_thread", &self.init_thread).finish()
    }
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
            resources: Vec::new()
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
        let class_index = self.global.class_index(JAVA_CLASS)?;

        let inner = RustObject::Class(ClassInstance::new(metadata));
        let class = Object::new(class_index, self.global.resolve_fields(class_index)?, inner)?;

        let object_index = self.global.new_object(class)?;

        self.clinit(object_index)?;
        self.default_init(class_index, object_index)?;

        Ok(object_index)
    }

    pub fn load_main_class(self: &mut Self) -> Result<usize, WasmJVMError> {
        // TODO: Look at Jar for actual main class.
        let main_class = "Main";
        let class_index = self.load_class_name(main_class)?;
        self.global.set_main_class(&main_class.to_string())?;

        Ok(class_index)
    }

    pub fn load_class_name(self: &mut Self, name: &str) -> Result<usize, WasmJVMError> {
        let class = self.extract_class(name)?;
        self.load_class(class)
    }

    fn extract_class(self: &mut Self, name: &str) -> Result<Class, WasmJVMError> {
        for resource in self.resources.iter_mut() {
            if let Ok(class) = resource.as_mut().load_class(name) {
                return Ok(class);
            }
        }

        Err(WasmJVMError::ClassNotFoundException(format!("Could not load class {}", name)))
    }

    pub fn load_jar<F: 'static + std::io::Read + std::io::Seek>(
        self: &mut Self,
        jar: Jar<F>,
    ) -> Result<(), WasmJVMError> {
        self.resources.push(Box::new(jar));

        Ok(())
    }

    fn extract_boot_class(self: &mut Self, name: &str) -> Result<Class, WasmJVMError> {
        if let Ok(class) = self.extract_class(name) {
            Ok(class)
        } else {
            Err(WasmJVMError::ClassNotFoundException(format!("Could not load boot class {}", name)))
        }
    }

    pub fn boot(self: &mut Self) -> Result<(), WasmJVMError> {
        self.load_boot_classes()?;
        self.load_main_class()?;

        Ok(())
    }

    fn load_boot_classes(self: &mut Self) -> Result<(), WasmJVMError> {
        let object_class = self.extract_boot_class(JAVA_OBJECT)?;
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

        let class_class = self.extract_boot_class(JAVA_CLASS)?;
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

        let loader_class = self.extract_boot_class(JAVA_LOADER)?;
        let loader = Object::new(
            class_index.clone(),
            class_fields.clone(),
            RustObject::Class(ClassInstance::new(loader_class)),
        )?;
        let loader_index = self.global.new_object(loader)?;
        clinits.push(loader_index);
        inits.push((class_index, loader_index));

        let thread_class = self.extract_boot_class(JAVA_THREAD)?;
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
}
