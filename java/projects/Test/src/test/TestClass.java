package test;

public class TestClass extends Test {
    private abstract class AbstractClass {
        public abstract void abstractMethod();

        public void parentMethod() {
            System.out.println("void parentMethod(): AbstractClass");
        }

        public void chainMethod() {
            System.out.println("void chainMethod(): AbstractClass");
        }
    }

    private interface Interface {
        public void interfaceMethod();
    }

    private class ExtendClass extends AbstractClass implements Interface {
        @Override
        public void abstractMethod() {
            System.out.println("abstract void abstractMethod(): ExtendClass");
        }

        @Override
        public void interfaceMethod() {
            System.out.println("void interfaceMethod(): ExtendClass");
        }

        @Override
        public void chainMethod() {
            super.chainMethod();
            System.out.println("void chainMethod(): ExtendClass");
        }
    }

    private class GenericClass<T> {
        private T t; 

        public GenericClass(T t) {
            this.t = t;
        }

        public void genericMethod() {
            System.out.println(new StringBuilder().append("T t: ").append(this.t.toString()).toString());
        }
    }

    private void testInheritance() {
        System.out.println("[Test Inheritance]");

        ExtendClass extendClass = new ExtendClass();
        AbstractClass abstractClass = extendClass;

        abstractClass.abstractMethod();
        extendClass.parentMethod();
    
        abstractClass.chainMethod();
        extendClass.chainMethod();
    }

    private void testReflection() {
        System.out.println("[Test Reflection]");

        AbstractClass abstractClass = new ExtendClass();
        System.out.println(new StringBuilder().append("Class: ").append(abstractClass.getClass().toString()).toString());
    }

    private void testGeneric() {
        System.out.println("[Test Generic]");

        GenericClass<String> genericClass = new GenericClass<String>("Test");
        genericClass.genericMethod();
    }

    private void testInterface() {
        System.out.println("[Test Interface]");

        Interface interfaceClass = new ExtendClass();
        interfaceClass.interfaceMethod();
    }

    private void testInstanceOf() {
        System.out.println("[Test InstanceOf]");

        Object extendClass = new ExtendClass();
        System.out.println(new StringBuilder().append("ExtendClass instanceof AbstractClass = ").append(extendClass instanceof AbstractClass).toString());
        System.out.println(new StringBuilder().append("ExtendClass instanceof Interface = ").append(extendClass instanceof Interface).toString());
        System.out.println(new StringBuilder().append("ExtendClass instanceof Test = ").append(extendClass instanceof Test).toString());
    }

    @Override
    public void run() {
        testInheritance();
        testReflection();
        testGeneric();
        testInterface();
        testInstanceOf();
    }
}
