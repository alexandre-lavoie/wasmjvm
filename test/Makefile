TARGET = $(wildcard base/*.java)
CORE = $(wildcard base/java/lang/*.java)

main:
	rm -rf ./dist
	echo $(SOURCES)
	javac -d dist -source 1.8 -target 1.8 -extdirs ./base -sourcepath ./base -classpath dist $(CORE) $(TARGET)
	jar cvfe Test.jar Main ./dist/*
