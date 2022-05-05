package test;

public class TestMath extends Test {
    private long factorial(long n) {
        if (n == 0) {
            return 1;
        } else {
            return n * factorial(n - 1);
        }
    }

    private void testFactorial() {
        System.out.println(new StringBuilder().append("Factorial(5) = ").append(factorial(5)).toString());
    }

    private boolean isPrime(int value) {
        for(int j = 2; j < value; j++) {
            if (value % j == 0) return false;
        }

        return true;
    }

    private String getPrimes(int max) {
        StringBuilder builder = new StringBuilder();

        for(int i = 2; i < max; i++) {
            if(isPrime(i)) {
                builder.append(i).append(",");
            }
        }

        return builder.toString();
    }

    private void testPrimes() {
        System.out.println(new StringBuilder().append("Primes: ").append(getPrimes(100)).toString());
    }

    private static int VALUE = 1;
    private static int NEQ_VALUE = 2;
    private static int COPY_VALUE = 1;

    private void testLogic() {
        System.out.println(new StringBuilder().append(VALUE).append(" < ").append(NEQ_VALUE).append(" = ").append(VALUE < NEQ_VALUE).toString());
        System.out.println(new StringBuilder().append(NEQ_VALUE).append(" < ").append(VALUE).append(" = ").append(NEQ_VALUE < VALUE).toString());

        System.out.println(new StringBuilder().append(VALUE).append(" <= ").append(NEQ_VALUE).append(" = ").append(VALUE <= NEQ_VALUE).toString());
        System.out.println(new StringBuilder().append(NEQ_VALUE).append(" <= ").append(VALUE).append(" = ").append(NEQ_VALUE <= VALUE).toString());
        System.out.println(new StringBuilder().append(VALUE).append(" <= ").append(COPY_VALUE).append(" = ").append(VALUE <= COPY_VALUE).toString());

        System.out.println(new StringBuilder().append(VALUE).append(" > ").append(NEQ_VALUE).append(" = ").append(VALUE > NEQ_VALUE).toString());
        System.out.println(new StringBuilder().append(NEQ_VALUE).append(" > ").append(VALUE).append(" = ").append(NEQ_VALUE > VALUE).toString());

        System.out.println(new StringBuilder().append(VALUE).append(" >= ").append(NEQ_VALUE).append(" = ").append(VALUE >= NEQ_VALUE).toString());
        System.out.println(new StringBuilder().append(NEQ_VALUE).append(" >= ").append(VALUE).append(" = ").append(NEQ_VALUE >= VALUE).toString());
        System.out.println(new StringBuilder().append(VALUE).append(" >= ").append(COPY_VALUE).append(" = ").append(VALUE >= COPY_VALUE).toString());

        System.out.println(new StringBuilder().append(VALUE).append(" == ").append(NEQ_VALUE).append(" = ").append(VALUE == NEQ_VALUE).toString());
        System.out.println(new StringBuilder().append(VALUE).append(" == ").append(COPY_VALUE).append(" = ").append(VALUE == COPY_VALUE).toString());

        System.out.println(new StringBuilder().append(VALUE).append(" != ").append(NEQ_VALUE).append(" = ").append(VALUE != NEQ_VALUE).toString());
        System.out.println(new StringBuilder().append(VALUE).append(" != ").append(COPY_VALUE).append(" = ").append(VALUE != COPY_VALUE).toString());
    }

    @Override
    public void run() {
        testPrimes();
        testFactorial();
        testLogic();
    }
}
