package test;

import java.io.*;
import java.util.*;

public class TestIO extends Test {
    private void testSystem() {
        System.out.println("[Test System]");
        System.out.print("Input: ");
        Scanner scanner = new Scanner(System.in);
        String input = scanner.nextLine();
        System.out.println(new StringBuilder().append("Output: ").append(input).toString());
    }

    private void testFile() {
        System.out.println("[Test File]");
        try {
            System.out.print("File: ");
            Scanner scanner = new Scanner(System.in);
            String path = scanner.nextLine();
    
            System.out.print("Input: ");
            String content = scanner.nextLine();
            PrintStream fileWriter = new PrintStream(path);
            fileWriter.print(content);

            Scanner fileScanner = new Scanner(new FileInputStream(path));
            System.out.println(new StringBuilder().append("Output: ").append(fileScanner.nextLine()).toString());
        } catch (Exception exception) {}
    }

    @Override
    public void run() {
        testSystem();
        testFile();
    }
}
