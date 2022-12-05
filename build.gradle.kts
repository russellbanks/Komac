import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    alias(libs.plugins.buildconfig)
    alias(libs.plugins.detekt)
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.ksp)
    application
}

group = "com.russellbanks"
version = "alpha"

repositories {
    mavenCentral()
}

dependencies {
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

    // KotlinX Serialization - https://github.com/Kotlin/kotlinx.serialization
    implementation(libs.kotlinx.serialization.json)

    // Ktor - https://github.com/ktorio/ktor
    implementation(libs.ktor.client.contentnegotiation)
    implementation(libs.ktor.client.core)
    implementation(libs.ktor.client.java)
    implementation(libs.ktor.serialization.kotlinx.json)

    // Mordant - https://github.com/ajalt/mordant
    implementation(libs.mordant)
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
