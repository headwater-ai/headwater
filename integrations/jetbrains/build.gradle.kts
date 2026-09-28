// SPDX-License-Identifier: Apache-2.0
//
// Compiles the plugin against the IntelliJ Platform and packages it. Run
// `gradle buildPlugin` in this directory with Gradle 9.1 or later on a JDK 21 or
// later. No wrapper jar is committed, because that is a binary in the tree.
//
// The client suite does not need this build: `java
// integrations/jetbrains/test/RunTests.java` compiles and runs it with a bare
// JDK. This build is what shows that the glue compiles against the platform.

plugins {
    java
    id("org.jetbrains.intellij.platform") version "2.19.0"
}

group = "ai.headwater"
version = "0.1.0"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        intellijIdeaCommunity("2024.3.6")
    }
}

tasks.withType<JavaCompile>().configureEach {
    options.release = 21
    options.encoding = "UTF-8"
    options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
}

intellijPlatform {
    buildSearchableOptions = false
    instrumentCode = false
    pluginConfiguration {
        ideaVersion {
            sinceBuild = "243"
        }
    }
}
