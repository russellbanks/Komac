import org.jetbrains.kotlin.gradle.tasks.KotlinCompile
import org.panteleyev.jpackage.ImageType
import java.util.Calendar

plugins {
    alias(libs.plugins.buildconfig)
    alias(libs.plugins.detekt)
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.ksp)
    alias(libs.plugins.shadow)
    id("org.panteleyev.jpackageplugin") version "1.5.0"
    application
}

group = "com.russellbanks"
version = "0.6.0"

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

    // Implementation - https://github.com/charleskorn/kaml
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
}

task("copyDependencies", Copy::class) {
    from(configurations.runtimeClasspath).into("$buildDir/jars")
}

task("copyJar", Copy::class) {
    from(tasks.jar).into("$buildDir/jars")
}

tasks.jpackage {
    dependsOn("build", "copyDependencies", "copyJar")

    input  = "$buildDir/jars"
    destination = "$buildDir/distributions"

    appName = project.name
    appVersion = project.version.toString()

    copyright = "Copyright (c) ${Calendar.getInstance().get(Calendar.YEAR)} Russell Banks"

    licenseFile = "$projectDir/src/main/resources/gpl-3.0.txt"

    vendor = "Russell Banks"

    mainJar = tasks.jar.get().archiveFileName.get()
    mainClass = "${project.group}.${application.mainClass}"

    javaOptions = listOf("-Dfile.encoding=UTF-8")

    windows {
        type = ImageType.MSI
        winConsole = true
        winUpgradeUuid = "2D35545F-9065-48C3-A345-42244A3E9FBF"
    }

    linux {
        type = ImageType.DEB
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

application {
    mainClass.set("MainKt")
}
