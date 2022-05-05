package java.util;

public class Random {
    private native long nativeNextLong();

    public long nextLong() {
        return nativeNextLong();
    }

    public long nextLong(long bound) {
        return nextLong() % bound;
    }

    public int nextInt() {
        int next = (int)nextLong();
        
        if (next < 0) next *= -1;

        return next;
    }

    public int nextInt(int bound) {
        return nextInt() % bound;
    }
}
