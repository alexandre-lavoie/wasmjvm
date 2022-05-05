use std::{
    collections::HashMap,
    io::{BufRead, Read, Write},
};

use wasmjvm_class::{Descriptor, MethodRef, SingleType, Type};
use wasmjvm_native::{
    async_box, register_method, NativeEnv, NativeInterface, Primitive, RustObject,
};

pub fn register(interface: &mut NativeInterface) {
    unsafe {
        STREAMS = Some(HashMap::new());
    }

    register_method!(
        interface,
        async_file_bind_read,
        "java/io/FileInputStream",
        "nativeBind",
        vec![],
        Type::Single(SingleType::Void)
    );

    register_method!(
        interface,
        async_file_read,
        "java/io/FileInputStream",
        "nativeRead",
        vec![],
        Type::Single(SingleType::Int)
    );

    register_method!(
        interface,
        async_file_bind_write,
        "java/io/FileOutputStream",
        "nativeBind",
        vec![],
        Type::Single(SingleType::Void)
    );

    register_method!(
        interface,
        async_file_write,
        "java/io/FileOutputStream",
        "nativeWrite",
        vec![Type::Single(SingleType::Int)],
        Type::Single(SingleType::Void)
    );

    register_method!(
        interface,
        async_random_long,
        "java/util/Random",
        "nativeNextLong",
        vec![],
        Type::Single(SingleType::Long)
    );
}

static mut STREAMS: Option<HashMap<usize, Box<dyn FileCursor>>> = None;

trait FileCursor {
    fn write(self: &mut Self, value: i32);
    fn read(self: &mut Self) -> i32;
}

struct SystemStream {
    buffer: Vec<u8>,
}

impl SystemStream {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn buffer_read(self: &mut Self) {
        std::io::stdout().flush().unwrap();
        let line = std::io::stdin().lock().lines().next().unwrap().unwrap();
        let mut bytes: Vec<u8> = line.into();
        bytes.push('\n' as u8);
        bytes.reverse();
        self.buffer.append(&mut bytes);
    }
}

impl FileCursor for SystemStream {
    fn write(self: &mut Self, value: i32) {
        std::io::stdout().lock().write(&[value as u8]).unwrap();
    }

    fn read(self: &mut Self) -> i32 {
        if self.buffer.is_empty() {
            self.buffer_read();
        }

        self.buffer.pop().unwrap() as i32
    }
}

struct FileStream {
    file: std::fs::File,
}

impl FileStream {
    pub fn new(path: String, is_read: bool) -> Self {
        let file = if is_read {
            std::fs::File::open(path).unwrap()
        } else {
            std::fs::File::create(path).unwrap()
        };

        Self { file }
    }
}

impl FileCursor for FileStream {
    fn write(self: &mut Self, value: i32) {
        self.file.write(&[value as u8]).unwrap();
    }

    fn read(self: &mut Self) -> i32 {
        let mut buffer = [0u8; 1];
        self.file.read(&mut buffer).unwrap();

        buffer[0] as i32
    }
}

fn file_bind_mode(env: &mut NativeEnv, is_read: bool) -> Primitive {
    if let [this_ref, ..] = &env.variables()[..] {
        if let Primitive::Reference(this_index) = this_ref {
            let this = env.reference(&this_ref).unwrap();
            let path_ref = this.fields.get("path").unwrap();
            let path_object = env.reference(&path_ref).unwrap();
            if let RustObject::String(path) = path_object.inner() {
                if let Some(streams) = unsafe { &mut STREAMS } {
                    if !streams.contains_key(this_index) {
                        let stream: Box<dyn FileCursor> = if path == "<sys>" {
                            Box::new(SystemStream::new())
                        } else {
                            Box::new(FileStream::new(path.to_string(), is_read))
                        };

                        streams.insert(*this_index, stream);

                        return Primitive::Void;
                    }
                }
            }
        }
    }

    unreachable!();
}

async_box!(async_file_bind_read, file_bind_read);
async fn file_bind_read(env: &mut NativeEnv) -> Primitive {
    file_bind_mode(env, true)
}

async_box!(async_file_bind_write, file_bind_write);
async fn file_bind_write(env: &mut NativeEnv) -> Primitive {
    file_bind_mode(env, false)
}

async_box!(async_file_read, file_read);
async fn file_read(env: &mut NativeEnv) -> Primitive {
    if let [this_ref, ..] = &env.variables()[..] {
        if let Primitive::Reference(this_index) = this_ref {
            if let Some(streams) = unsafe { &mut STREAMS } {
                return Primitive::Int(streams.get_mut(this_index).unwrap().read());
            }
        }
    }

    unreachable!()
}

async_box!(async_file_write, file_write);
async fn file_write(env: &mut NativeEnv) -> Primitive {
    if let [this_ref, value, ..] = &env.variables()[..] {
        if let Primitive::Reference(this_index) = this_ref {
            if let Some(streams) = unsafe { &mut STREAMS } {
                if let Primitive::Int(value) = value {
                    streams.get_mut(this_index).unwrap().write(*value);
                    return Primitive::Void;
                }
            }
        }
    }

    unreachable!()
}

async_box!(async_random_long, random_long);
async fn random_long(_env: &mut NativeEnv) -> Primitive {
    Primitive::Long(rand::random::<i64>().abs())
}
