import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar
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
version = "0.7.0"

repositories {
    mavenCentral()
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

    // Mordant - https://github.com/ajalt/mordant
    implementation(libs.mordant)

    // Secure Token Storage Library - https://github.com/microsoft/vsts-authentication-library-for-java
    implementation(libs.auth.secure.storage)

    // SLF4J No-operation implementation - https://github.com/qos-ch/slf4j
    implementation(libs.slf4j.nop)
}

task("copyDependencies", Copy::class) {
    from(configurations.runtimeClasspath).into("$buildDir/jars")
}

task("copyJar", Copy::class) {
    from(tasks.jar).into("$buildDir/jars")
}

application {
    mainClass.set("MainKt")
}

tasks.jpackage {
    dependsOn("build", "copyDependencies", "copyJar")
    input  = "$buildDir/jars"
    destination = "$buildDir/distributions"
    /* addModules = listOf(
        "java.base", "java.desktop", "java.logging", "java.management", "java.net.http", "java.sql", "java.xml"
    ) */
    resourceDir = "$rootDir/config/wix"
    appName = project.name
    appVersion = project.version.toString()
    copyright = "Copyright (c) Russell Banks"
    licenseFile = "$projectDir/src/main/resources/gpl-3.0.txt"
    vendor = "Russell Banks"
    mainJar = tasks.jar.get().archiveFileName.get()
    mainClass = application.mainClass.get()
    javaOptions = listOf("-Dfile.encoding=UTF-8")

    windows {
        winPerUserInstall = true
        type = ImageType.MSI
        winConsole = true
        winUpgradeUuid = "2D35545F-9065-48C3-A345-42244A3E9FBF"
    }

    linux {
        type = ImageType.DEB
    }
}

tasks.withType<ShadowJar> {
    minimize {
        exclude(dependency(libs.jna.asProvider().get().toString()))
        exclude(dependency(libs.slf4j.nop.get().toString()))
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
    buildConfigField("String", "appVersion", provider { "\"${project.version}\"" })
}

tasks.withType<KotlinCompile> {
    kotlinOptions.jvmTarget = JavaVersion.VERSION_17.toString()
}
