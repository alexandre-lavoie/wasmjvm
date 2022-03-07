package java.lang;

public class System {
    public static void println(String string) {
        print(string.append("\n"));
    }

    public static native void print(String string);

    public static String prompt(String string) {
        println(string);
        return input();
    }

    public static native String input();
}
