import * as React from "react";

export default class Interface {
    public static setOutput: React.Dispatch<React.SetStateAction<string>>;
    
    private static output: string = "";

    public static log(message: string): void {
        Interface.output += message;
        Interface.setOutput(Interface.output);

        console.log(Interface.output);
    }

    public static error(message: string): void {
        console.error(message);
    }

    public static prompt(): string {
        let output = prompt("Input");

        this.log(output + "\n");

        return output;
    }
}
