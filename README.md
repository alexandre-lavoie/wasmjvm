# ☕ WasmJVM

WasmJVM is a WebAssembly-friendly implementation of the [Java Virtual Machine (JVM)](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-1.html). The primary goal is to create a WebAssembly VM that follows closely the "Write once, run anywhere" (WORA) philosophy. Imagine running a `.jar` on the web like JavaScript. Porting [Java Swing](https://en.wikipedia.org/wiki/Swing_(Java)) and [LWJGL](https://www.lwjgl.org/) (or other OpenGL/OpenCL libraries) to WebGL is also considered.

## 🚩 Getting Started

We still are in development. Eventually this section will include a simple JavaScript import and NPM/Yarn install. For now, you will have to build from source...

## ⚒️ Building

This section includes all you need to build the project! We are currently developing on [Arch Linux](https://archlinux.org/)s. We can't sadly confirm if this works on other platforms yet...

### 🔗 Depedencies

- [OpenJDK](https://openjdk.java.net/) / etc (Build `.java` to `.class`/`.jar`).
- [Rust](https://www.rust-lang.org/)

### 🖥️ OS Build

This is not implemented as of writing this.

### 🌎 Browser Build

This is not implemented as of writing this.

## 📅 Milestones

The following is the currently planned features for the project. This may change according to the interest/difficulties over time.

### 🔌 On The Web

Create the most basic program to run on the web. This should be simply returning a primitive from a single method class.

- [ ] OS Build.
- [ ] Web Build.
- [ ] Handle single `.class` file.
- [ ] Basic VM.
- [ ] Basic JavaScript interactions.

### 🍖 Primitive

Create a program that can handle all primitive operations. 

- [ ] Byte
- [ ] Char
- [ ] Double
- [ ] Int
- [ ] Long
- [ ] Object
- [ ] Short
- [ ] Boolean
- [ ] Array

### 👋 Hello World

Create a program that can handle non-primitive types. Output should be printable to screen using `System.out.println` (or equivalent).

- [ ] Handle `.jar` file.
- [ ] Multiple classes.
- [ ] [Native](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-2.html#jvms-2.5.6) methods.

## 🔖 Related Projects

This project is not a unique idea. Other briliant teams are also trying to solve this issue - you might be interested in their approach instead.

- [javaemu](https://github.com/poruruba/javaemu): Emscripten WebAssembly JVM.

- [jvm](https://github.com/douchuan/jvm): Rust JVM.

- [TeamVM](https://www.teavm.org/) / [Bytecoder](https://github.com/mirkosertic/Bytecoder) / [CheerpJ](https://github.com/leaningtech/cheerpj-meta) / [JWebAssembly](https://github.com/i-net-software/JWebAssembly): Transpile Java bytecode to JavaScript/Webassembly.

- [jsjvm](https://gitlab.com/neoexpert/jvm/-/tree/master/jsjvm): JavaScript JVM.

## 👥 Contributing

If this project seems interesting to you, leave a star and open an issue! We are unsure of the interest therefore we will not open extra communication avenues until necessary. 

## 📄 License

The core JVM implementation is written from scratch. All current code and future code made by us will be under the [MIT License](./LICENSE). We will most likely rely on [OpenJDK](https://openjdk.java.net/) for the core Java classes in the near future. We will try to make the code as modular as possible so you can switch out as necessary. All additions will be licensed here accordingly.
