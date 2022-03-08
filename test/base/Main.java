public class Main {
    public static void main() {
        String name = System.prompt("Enter your name: ");

        String message = "Hello ".append(name).append("! How are you?");
        System.println(message);

        message = "(".append(name.getClass().toString()).append(")");
        System.println(message);
    }
}
