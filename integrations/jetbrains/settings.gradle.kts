// SPDX-License-Identifier: Apache-2.0

plugins {
    // The IntelliJ Platform Gradle Plugin compiles with a JDK 21 toolchain,
    // the one IntelliJ Platform 2024.3 runs on. This resolver downloads that
    // JDK when the machine has none, so a host with only a newer JDK builds.
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

rootProject.name = "headwater-jetbrains"
