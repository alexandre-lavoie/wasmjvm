import * as React from "react";

abstract class Stream {
    public abstract read(pointer: number): number;
    public abstract write(pointer: number, value: number);
}

export class SystemStream extends Stream {
    private input: number[] = [];

    public read(): number {
        if(this.input.length == 0) {
            let input = prompt("Input") + "\n";
            Interface.print(input);
            this.input = [...new TextEncoder().encode(input)];
        }

        return this.input.shift();
    }

    public write(value: number) {
        Interface.print(String.fromCharCode(value));
    }
}

export class FileStream extends Stream {
    private path: string;
    private pointer: number;

    public constructor(path: string) {
        super();
        this.path = path;
        this.pointer = 0;
    }

    private path_tag(path: string): string {
        return `path-${path}`;
    }

    public read(): number {
        let value = localStorage.getItem(this.path_tag(this.path));

        if(value == null) return 0;
        if(this.pointer >= value.length) return 0;

        return value.charCodeAt(this.pointer++);
    }

    public write(value: number) {
        let stored = localStorage.getItem(this.path_tag(this.path)) || "";
        localStorage.setItem(this.path_tag(this.path), stored + String.fromCharCode(value));
    }
}

export default class Interface {
    public static setOutput: React.Dispatch<React.SetStateAction<string>>;

    private static streams: Map<number, Stream> = new Map<number, Stream>();
    private static output: string = "";

    public static file_bind(pointer: number, path: string): void {
        if(path == "<sys>") {
            this.streams[pointer] = new SystemStream();
        } else {
            this.streams[pointer] = new FileStream(path);
        }
    }

    public static file_read(pointer: number): number {
        return this.streams[pointer].read(pointer);
    }

    public static file_write(pointer: number, value: number): void {
        this.streams[pointer].write(value);
    }

    public static print(str: string): void {
        Interface.output += str;
        Interface.setOutput(Interface.output);
    }

    public static error(message: string): void {
        console.error(message);
    }
}
