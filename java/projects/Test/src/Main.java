import java.io.*;
import java.util.*;
import test.*;

public class Main {
    public static void main(String[] args) {
        System.out.print("Test: ");
        Scanner scanner = new Scanner(System.in);
        String testName = scanner.nextLine();

        Test test = null;
        if(testName.equals("io")) {
            test = new TestIO();
        } else if(testName.equals("class")) {
            test = new TestClass();
        } else if(testName.equals("math")) {
            test = new TestMath();
        }

        if(test == null) {
            System.out.println(new StringBuilder().append("Could not find test ").append(testName).append(".").toString());
        } else {
            test.run();
        }
    }
}
