import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar
import org.jetbrains.kotlin.gradle.plugin.mpp.pm20.util.distsDirectory
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile
import org.panteleyev.jpackage.ImageType

plugins {
    alias(libs.plugins.buildconfig)
    alias(libs.plugins.detekt)
    alias(libs.plugins.jpackage)
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.ksp)
    alias(libs.plugins.shadow)
    application
}

group = "com.russellbanks"
version = "1.0.5"

repositories {
    mavenCentral()
    maven("https://oss.sonatype.org/content/repositories/snapshots/")
}

dependencies {
    // Clikt - https://github.com/ajalt/clikt
    implementation(libs.clikt)

    // GitHub API - https://github.com/hub4j/github-api
    implementation(libs.github.api)

    // Kotlin Coroutines - https://github.com/Kotlin/kotlinx.coroutines
    implementation(libs.coroutines.core)

    // Detekt Formatting Plugin - https://github.com/detekt/detekt
    detektPlugins(libs.detekt.formatting)

    // JLine - https://github.com/jline/jline3
    implementation(libs.jline.terminal.jna)

    // JNA - https://github.com/java-native-access/jna
    implementation(libs.jna)
    implementation(libs.jna.platform)

    // Kaml - https://github.com/charleskorn/kaml
    implementation(libs.kaml)

    // Koin - https://github.com/InsertKoinIO/koin
    implementation(libs.koin.core)
    implementation(libs.koin.annotations)
    ksp(libs.koin.ksp.compiler)

    // Kotest - https://github.com/kotest/kotest
    testImplementation(libs.kotest.junit5)
    testImplementation(libs.kotest.assertions.core)
    testImplementation(libs.kotest.framework.datatest)
    testImplementation(libs.kotest.extensions.assertions.ktor)

    // KotlinX Serialization - https://github.com/Kotlin/kotlinx.serialization
    implementation(libs.kotlinx.serialization.json)

    // Ktor - https://github.com/ktorio/ktor
    implementation(libs.ktor.client.core)
    implementation(libs.ktor.client.java)
    implementation(libs.ktor.serialization.kotlinx.json)

    // Mockk - https://github.com/mockk/mockk
    testImplementation(libs.mockk)

    // Mordant - https://github.com/ajalt/mordant
    implementation(libs.mordant)

    // Skrape{it} - https://github.com/skrapeit/skrape.it
    implementation(libs.skrapeit.htmlparser) {
        constraints {
            implementation(libs.jsoup)
        }
    }

    // SLF4J No-operation implementation - https://www.slf4j.org
    implementation(libs.slf4j.nop)
}

application {
    applicationDefaultJvmArgs = listOf("-Dfile.encoding=UTF-8")
    mainClass.set("MainKt")
}

tasks.withType<ShadowJar> {
    minimize {
        exclude(dependency(libs.jna.asProvider().get().toString()))
        exclude(dependency(libs.slf4j.nop.get().toString()))
        exclude(dependency(libs.jline.terminal.asProvider().get().toString()))
        exclude(dependency(libs.jline.terminal.jna.get().toString()))
    }
}

tasks.jpackage {
    dependsOn("build")
    input  = tasks.shadowJar.get().destinationDirectory.get().toString()
    destination = distsDirectory.get().toString()
    addModules = listOf(
        "java.base",
        "java.desktop",
        "java.instrument",
        "java.management",
        "java.net.http",
        "java.sql",
        "jdk.crypto.cryptoki",
        "jdk.unsupported"
    )
    resourceDir = "$rootDir/config/wix"
    appName = project.name
    appVersion = project.version.toString()
    copyright = "Copyright (c) Russell Banks"
    licenseFile = "$projectDir/assets/gpl-3.0.rst"
    vendor = "Russell Banks"
    mainJar = tasks.shadowJar.get().archiveFileName.get()
    mainClass = application.mainClass.get()
    javaOptions = listOf("-Dfile.encoding=UTF-8")
    jpackageEnvironment = mapOf(
        "BrandingDialog" to "$resourceDir/dialog.bmp",
        "BrandingBanner" to "$resourceDir/banner.bmp"
    )

    windows {
        icon = "$projectDir/assets/logo.ico"
        winPerUserInstall = System.getenv("PER_USER_INSTALL")?.toBooleanStrictOrNull() ?: true
        winDirChooser = true
        type = ImageType.EXE
        winConsole = true
        winUpgradeUuid = "2D35545F-9065-48C3-A345-42244A3E9FBF"
    }

    linux {
        icon = "$projectDir/assets/logo.png"
        type = ImageType.DEB
    }

    mac {
        icon = "$projectDir/assets/logo.icns"
        type = ImageType.DMG
    }
}

tasks.withType<Test>().configureEach {
    useJUnitPlatform()
}

sourceSets.main {
    kotlin.srcDirs("build/generated/ksp/main/kotlin")
}

buildConfig {
    buildConfigField("String", "appName", "\"${project.name}\"")
    buildConfigField("String", "appVersion", "\"${project.version}\"")
}

tasks.withType<KotlinCompile> {
    kotlinOptions.jvmTarget = JavaVersion.VERSION_17.toString()
}
