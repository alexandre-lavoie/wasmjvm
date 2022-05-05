package game;

import java.util.Scanner;
import java.util.Random;

public class Hangman extends Game {
    private static final String[] WORDS = new String[] {
        "java",
        "jvm",
        "coffee",
        "cup",
        "doughnut",
        "recursion",
        "variable",
        "integer"
    };

    // Taken from: https://gist.github.com/chrishorton/8510732aa9a80a03c829b09f12e20d9c
    private static final String[] DRAWINGS = new String[]{
        "\n +---+\n |   |\n     |\n     |\n     |\n     |\n=========\n",
        "\n +---+\n |   |\n O   |\n     |\n     |\n     |\n=========\n",
        "\n +---+\n |   |\n O   |\n |   |\n     |\n     |\n=========\n",
        "\n +---+\n |   |\n O   |\n/|   |\n     |\n     |\n=========\n",
        "\n +---+\n |   |\n O   |\n/|\\  |\n     |\n     |\n=========\n",
        "\n +---+\n |   |\n O   |\n/|\\  |\n/    |\n     |\n=========\n",
        "\n +---+\n |   |\n O   |\n/|\\  |\n/ \\  |\n     |\n=========\n"
    };

    private int state = 0;
    private String word;
    private byte[] wordBytes;
    private byte[] guesses;
    private Scanner scanner;

    public Hangman() {
        this(WORDS[new Random().nextInt(WORDS.length)]);
    }

    public Hangman(String word) {
        this.word = word;
        this.wordBytes = word.getBytes();
        this.guesses = new byte[word.length()];
        this.scanner = new Scanner(System.in);
    }

    public int tick() {
        this.draw();

        byte guess = (byte)this.prompt();

        boolean valid = false;
        boolean won = true;

        for(int i = 0; i < wordBytes.length; i++) {
            if(wordBytes[i] == guess) {
                guesses[i] = guess;
                valid = true;
            }

            if (guesses[i] == '\0') {
                won = false;
            }
        }

        if(won) {
            return 1;
        }

        if(!valid) {
            if(++this.state >= DRAWINGS.length - 1) {
                return 2;
            }
        }

        return 0;
    }

    public void draw() {
        drawHangman(this.state);
        drawGuess();
    }

    public void drawEnd() {
        drawHangman(this.state);
        System.out.println(this.word);
        System.out.println();
    }

    private void drawHangman(int state) {
        System.out.println(DRAWINGS[state]);
    }

    private void drawGuess() {
        StringBuilder builder = new StringBuilder();

        for(int i = 0; i < guesses.length; i++) {
            if(guesses[i] == 0) {
                builder.append((char)'_');
            } else {
                builder.append((char)guesses[i]);
            }

            builder.append((char)' ');
        }
        builder.append((char)'\n');

        System.out.println(builder.toString());
    }

    private char prompt() {
        System.out.print("Guess: ");
        String line = this.scanner.nextLine();
        return line.charAt(0);
    }

    @Override
    public void run() {
        loop:
        while(true) {
            switch(this.tick()) {
                case 1:
                    this.drawEnd();
                    System.out.println("You win!");
                    break loop;
                case 2:
                    this.drawEnd();
                    System.out.println("You lose...");
                    break loop;
                default:
                    break;
            }
        }
    }
}
