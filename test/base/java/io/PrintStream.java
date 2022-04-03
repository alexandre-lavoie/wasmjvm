package java.io;

public class PrintStream {
    public PrintStream() {}

    public static native void print(String string);
    public void println(String string) {
        print(string.append("\n"));
    }
}
