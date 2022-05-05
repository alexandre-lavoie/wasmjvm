package java.util;

import java.io.*;

public class Scanner {
    private InputStream inputStream;

    public Scanner(InputStream inputStream) {
        this.inputStream = inputStream;
    }

    public String nextLine() {
        StringBuilder stringBuilder = new StringBuilder();

        while(true) {
            int next = inputStream.read();
            
            if(next == '\0' || next == '\n') {
                break;
            } else {
                stringBuilder.append((char)next);
            }
        }

        return stringBuilder.toString();
    }
}
