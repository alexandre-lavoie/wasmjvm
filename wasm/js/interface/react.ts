import * as React from "react";
import { JAVA_RESOURCES } from "./resources";

let worker = new Worker(new URL("./worker.ts", import.meta.url));

interface IBind {
    name: "bind",
    payload: {
        pointer: number,
        path: string
    }
}

interface IInput {
    name: "input",
    payload: {
        pointer: number
    }
}

interface IOutput {
    name: "output",
    payload: {
        pointer: number,
        buffer: string
    }
}

interface IRun {
    name: "run",
    payload: string
}

type IPayload = IBind | IInput | IOutput | IRun;

worker.onmessage = async ({ data }: { data: IPayload }) => {
    if (data.name == "bind") {
        ReactInterface.bind(data.payload.pointer, data.payload.path);
    } else if(data.name == "input") {
        worker.postMessage({ name: "input", payload: {
            pointer: data.payload.pointer,
            buffer: await ReactInterface.input(data.payload.pointer)
        }});
    } else if(data.name == "output") {
        ReactInterface.output(data.payload.pointer, data.payload.buffer);
    } else if(data.name == "run") {
        ReactInterface.setRunning(false);
        console.log(data.payload);
    }
}

abstract class Stream {
    public abstract read(): Promise<string>;
    public abstract write(value: string): Promise<void>;
}

export class SystemStream extends Stream {
    private static output: string = "";

    public async read(): Promise<string> {
        let buffer = await new Promise<string>(resolve => {
            ReactInterface.pendingStdins.push(resolve);
        });

        this.write(buffer);

        return buffer;
    }

    public async write(buffer: string) {
        SystemStream.output += buffer;
        ReactInterface.setOutput(SystemStream.output);
    }
}

export class FileStream extends Stream {
    private path: string;

    public constructor(path: string) {
        super();
        this.path = path;
    }

    public async read(): Promise<string> {
        return await (await fetch(this.path)).text();
    }

    public async write(value: string) {
        // TODO
    }
}

export default class ReactInterface {
    public static setOutput: React.Dispatch<React.SetStateAction<string>>;
    public static setRunning: React.Dispatch<React.SetStateAction<boolean>>;
    public static setDev: React.Dispatch<React.SetStateAction<boolean>>;

    private static streams: Map<number, Stream> = new Map();
    public static pendingStdins: ((buffer: string) => void)[] = [];

    public static loadJar(jar: Uint8Array) {
        worker.postMessage({name: "loadJar", payload: jar});
    }

    public static run() {
        ReactInterface.setRunning(true);
        worker.postMessage({name: "run"});
    }

    public static bind(pointer: number, path: string) {
        if(path == "<sys>") {
            this.streams[pointer] = new SystemStream();
        } else {
            this.streams[pointer] = new FileStream(path);
        }
    }

    public static output(pointer: number, buffer: string) {
        this.streams[pointer].write(buffer);
    }

    public static async input(pointer: number): Promise<string> {
        return await this.streams[pointer].read();
    }

    public static stdin(message: string) {
        this.pendingStdins.forEach(stdin => stdin(message + "\n"));
        this.pendingStdins = [];
    }

    public static async loadResources() {
        if(JAVA_RESOURCES != null && JAVA_RESOURCES.jars != null) {
            setTimeout(async () => {
                let loadCount = 0;

                await Promise.all(JAVA_RESOURCES.jars.map(async (path: string) => {
                    try {
                        let res = await fetch(path);
                        let blob = await res.blob();
                        let buffer = await blob.arrayBuffer();
                        this.loadJar(new Uint8Array(buffer));
                        loadCount += 1;
                    } catch(e) {
                        console.error(e);
                    }
    
                    return true;
                }));
    
                if(loadCount == 0) {
                    this.setDev(true);
                }
            }, 1000);
        }
    }
}