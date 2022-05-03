import java.io.*;

public class Main {
    private static long factorial(long n) {
        if (n == 0) {
            return 1;
        } else {
            return n * factorial(n - 1);
        }
    }

    private static void testMath() {
        System.out.println(new StringBuilder().append(factorial(5)).toString());
    }

    private static void testLogic() {
        int a = 1;
        int b = 2;
        int c = 1;

        System.out.println(new StringBuilder().append(a).append(" < ").append(b).append(" -> ").append(a < b).toString());
        System.out.println(new StringBuilder().append(b).append(" < ").append(a).append(" -> ").append(b < a).toString());

        System.out.println(new StringBuilder().append(a).append(" <= ").append(b).append(" -> ").append(a <= b).toString());
        System.out.println(new StringBuilder().append(b).append(" <= ").append(a).append(" -> ").append(b <= a).toString());
        System.out.println(new StringBuilder().append(a).append(" <= ").append(c).append(" -> ").append(a <= c).toString());

        System.out.println(new StringBuilder().append(a).append(" > ").append(b).append(" -> ").append(a > b).toString());
        System.out.println(new StringBuilder().append(b).append(" > ").append(a).append(" -> ").append(b > a).toString());

        System.out.println(new StringBuilder().append(a).append(" >= ").append(b).append(" -> ").append(a >= b).toString());
        System.out.println(new StringBuilder().append(b).append(" >= ").append(a).append(" -> ").append(b >= a).toString());
        System.out.println(new StringBuilder().append(a).append(" >= ").append(c).append(" -> ").append(a >= c).toString());

        System.out.println(new StringBuilder().append(a).append(" == ").append(b).append(" -> ").append(a == b).toString());
        System.out.println(new StringBuilder().append(b).append(" == ").append(a).append(" -> ").append(b == a).toString());

        System.out.println(new StringBuilder().append(a).append(" != ").append(b).append(" -> ").append(a != b).toString());
        System.out.println(new StringBuilder().append(b).append(" != ").append(a).append(" -> ").append(b != a).toString());
    }

    private static void testInput() {
        System.out.print("Enter input: ");
        Scanner scanner = new Scanner(System.in);
        String input = scanner.nextLine();
        System.out.println(new StringBuilder().append("Output: ").append(input).toString());
    }

    private static void testReflection() {
        Child alex = new Child("Alex", 22);
        Child bob = new Child("Bob", 22);
    }

    private static void testFile() {
        System.out.print("Enter file: ");
        Scanner scanner = new Scanner(System.in);
        String path = scanner.nextLine();

        System.out.print("Enter content: ");
        String content = scanner.nextLine();
        FileOutputStream fileWriter = new FileOutputStream(path);
        fileWriter.print(content);

        Scanner fileScanner = new Scanner(new FileInputStream(path));
        System.out.println(new StringBuilder().append("File: ").append(fileScanner.nextLine()).toString());
    }

    public static void main(String[] args) {
        testFile();
    }
}
