import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    alias(libs.plugins.buildconfig)
    alias(libs.plugins.conveyor)
    alias(libs.plugins.detekt)
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.ksp)
    alias(libs.plugins.shadow)
    application
}

group = "com.russellbanks"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    // Clikt - https://github.com/ajalt/clikt
    implementation(libs.clikt)

    // Commons IO - https://commons.apache.org/proper/commons-io/
    implementation(libs.commons.io)

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

tasks.withType<Test>().configureEach {
    useJUnitPlatform()
}

sourceSets.main {
    java.srcDirs("build/generated/ksp/main/kotlin")
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

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
        vendor.set(JvmVendorSpec.ADOPTIUM)
    }
}
