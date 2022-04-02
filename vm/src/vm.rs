use wasmjvm_class::Class;
use wasmjvm_common::WasmJVMError;
use wasmjvm_native::{
    Global, Loader, NativeInterface, Primitive, RegisterFn, RustObject, Thread, ThreadResult,
    JAVA_CLASS, JAVA_LOADER, JAVA_NATIVE, JAVA_OBJECT, JAVA_THREAD,
};

pub struct VM {
    global: Global,
    classes: Vec<Class>,
    natives: Vec<RegisterFn>,
    initialized: bool,
    main_class: Option<String>,
}

impl VM {
    pub fn new() -> Self {
        let global = Global::new();
        Self {
            global,
            classes: Vec::new(),
            natives: Vec::new(),
            initialized: false,
            main_class: None,
        }
    }

    fn load_loader(self: &mut Self, cores: Vec<Class>) -> Result<(), WasmJVMError> {
        let loader = Loader::new(self.global.clone(), cores)?;

        self.global
            .new_rust_instance(&JAVA_LOADER.to_string(), RustObject::Loader(loader))?;

        Ok(())
    }

    pub fn main_class_set(self: &mut Self, class_name: &String) -> Result<(), WasmJVMError> {
        self.main_class = Some(class_name.clone());

        Ok(())
    }

    pub fn load_class(self: &mut Self, class: Class) -> Result<(), WasmJVMError> {
        self.classes.push(class);

        Ok(())
    }

    pub fn load_classes(self: &mut Self) -> Result<(), WasmJVMError> {
        if self.initialized {
            return Ok(());
        }

        let mut cores = Vec::new();
        let mut posts = Vec::new();

        while self.classes.len() > 0 {
            if let Some(class) = self.classes.pop() {
                let this_class = class.this_class().as_str();

                if this_class == JAVA_OBJECT
                    || this_class == JAVA_CLASS
                    || this_class == JAVA_THREAD
                    || this_class == JAVA_LOADER
                {
                    cores.push(class);
                } else {
                    posts.push(class);
                }
            } else {
                unreachable!()
            }
        }

        self.load_loader(cores)?;

        let loader = self.global.loader_mut()?;
        for class in posts {
            loader.load_class(class)?;
        }

        Ok(())
    }

    pub fn init_interface(self: &mut Self) -> Result<(), WasmJVMError> {
        if self.initialized {
            return Ok(());
        }

        let native = NativeInterface::new();
        self.global
            .new_rust_instance(&JAVA_NATIVE.to_string(), RustObject::Native(native))?;

        Ok(())
    }

    pub fn register_native(self: &mut Self, r#fn: RegisterFn) -> Result<(), WasmJVMError> {
        self.natives.push(r#fn);
        Ok(())
    }

    pub fn register_natives(self: &mut Self) -> Result<(), WasmJVMError> {
        while self.natives.len() > 0 {
            let r#fn = self.natives.pop().unwrap();
            self.global.register_native(r#fn)?;
        }

        Ok(())
    }

    pub fn unwind(self: &mut Self) -> Result<String, WasmJVMError> {
        let mut buffer = Vec::new();

        for thread_index in self.global.threads().clone().iter() {
            buffer.push(self.global.thread_mut(*thread_index)?.unwind()?);
        }

        Ok(buffer.join("\n"))
    }

    pub fn run(self: &mut Self) -> Result<Primitive, WasmJVMError> {
        self.initialized = true;

        if let Some(main_class) = &self.main_class {
            self.global.main_class_set(&main_class)?;
        } else {
            return Err(WasmJVMError::ClassNotFoundException(format!(
                "Main class not set"
            )));
        }

        let mut result = Primitive::Void;

        let mut main_thread = Thread::new(self.global.clone());
        main_thread.init_main()?;
        self.global
            .new_rust_instance(&JAVA_THREAD.to_string(), RustObject::Thread(main_thread))?;

        let (loader_clinit, loader_init) = self.global.loader()?.threads();
        loop {
            loop {
                match self.global.thread_tick(loader_clinit) {
                    Ok(ThreadResult::Result(..)) => break,
                    Ok(ThreadResult::Stop) => break,
                    Err(err) => return Err(err),
                    _ => {}
                }
            }

            loop {
                match self.global.thread_tick(loader_init) {
                    Ok(ThreadResult::Result(..)) => break,
                    Ok(ThreadResult::Stop) => break,
                    Err(err) => return Err(err),
                    _ => {}
                }
            }

            let mut stop = true;
            for thread_index in self.global.threads().clone().iter() {
                if *thread_index == loader_init || *thread_index == loader_clinit {
                    continue;
                }

                match self.global.thread_tick(*thread_index) {
                    Ok(ThreadResult::Continue) => {
                        stop = false;
                    }
                    Ok(ThreadResult::Stop) => {}
                    Ok(ThreadResult::Result(value)) => {
                        result = value;
                    }
                    Err(err) => return Err(err),
                }
            }

            if stop {
                break;
            }
        }

        Ok(result)
    }
}
