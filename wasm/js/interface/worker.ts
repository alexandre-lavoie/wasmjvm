import * as wasmJVM from "wasmjvm";
import RustInstance from "./rust";

interface ILoadJar {
    name: "loadJar",
    payload: Uint8Array
}

interface IRun {
    name: "run"
}

interface IStdin {
    name: "input",
    payload: {
        pointer: number,
        buffer: string
    }
}

type IPayload = ILoadJar | IRun | IStdin;

self.onmessage = async ({ data }: { data: IPayload }) => {
    if (data.name == "loadJar") {
        console.log(wasmJVM.load_jar(data.payload));
    } else if (data.name == "run") {
        let output = await wasmJVM.run();
        self.postMessage({ name: "run", payload: output });
    } else if (data.name == "input") {
        RustInstance.resolveStream(data.payload.pointer, data.payload.buffer);
    }
}
