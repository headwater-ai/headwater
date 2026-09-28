// SPDX-License-Identifier: Apache-2.0
//
// Compiles the client and its suite, and runs the suite. It needs a JDK 22 or
// later and nothing else: no Gradle, no JUnit, no IntelliJ Platform, and no
// network. Run it from the repository root:
//
//     java integrations/jetbrains/test/RunTests.java
//
// The source launcher runs this one file. A host with a JRE-only `java` and no
// `javac` still carries the compiler as the `jdk.compiler` module, so this file
// compiles `Client.java`, `ClientTest.java` and `FakeServer.java` into a
// temporary directory with `javax.tools`, loads `ClientTest` from there and
// calls its `main`. The glue classes are not compiled here, because they import
// the IntelliJ Platform; `build.gradle.kts` compiles them.

import java.io.IOException;
import java.lang.reflect.InvocationTargetException;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import javax.tools.JavaCompiler;
import javax.tools.ToolProvider;

public class RunTests {
    public static void main(String[] args) throws IOException, ReflectiveOperationException {
        Path here = locate();
        Path plugin = here.getParent();
        Path client = plugin.resolve("src/main/java/ai/headwater/jetbrains/Client.java");
        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        if (compiler == null) {
            System.err.println("RunTests: this java has no jdk.compiler module; run the suite with a JDK 22 or later");
            System.exit(2);
        }
        Path classes = Files.createTempDirectory("hw-jetbrains-classes-");
        List<String> argv = List.of(
                "-d", classes.toString(),
                "-encoding", "UTF-8",
                "-Xlint:all", "-Werror",
                client.toString(),
                here.resolve("ClientTest.java").toString(),
                here.resolve("FakeServer.java").toString());
        int status = compiler.run(null, null, null, argv.toArray(String[]::new));
        if (status != 0) {
            System.err.println("RunTests: the client or its suite does not compile");
            System.exit(status);
        }
        System.setProperty("hw.classes", classes.toString());
        System.setProperty("hw.test", here.toString());
        try (URLClassLoader loader = new URLClassLoader(new URL[] {classes.toUri().toURL()}, RunTests.class.getClassLoader())) {
            Class<?> suite = loader.loadClass("ClientTest");
            try {
                suite.getMethod("main", String[].class).invoke(null, (Object) args);
            } catch (InvocationTargetException e) {
                throw new RuntimeException(e.getCause());
            }
        }
    }

    // The directory that holds this file: `integrations/jetbrains/test` under
    // the current directory, which must be the repository root.
    private static Path locate() {
        Path here = Path.of("integrations/jetbrains/test").toAbsolutePath().normalize();
        if (!Files.isRegularFile(here.resolve("ClientTest.java"))) {
            System.err.println("RunTests: run this from the repository root; " + here + " holds no ClientTest.java");
            System.exit(2);
        }
        return here;
    }
}
