public class Main {
    public static void main() {
        String name = System.prompt("Enter your name: ");
        long id = name.getId();
        String idString = new String(id);
        String message = "Hello ".append(name).append(" (").append(idString).append(").");

        System.println(message);
    }
}
