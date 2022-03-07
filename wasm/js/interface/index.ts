export default class Interface {
    public static log(message: string): void {
        var log_output = message;
        console.log(message);
    }

    public static prompt(message: string): string {
        return prompt(message);
    }
}
