import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    alias(libs.plugins.detekt)
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.ksp)
    application
}

group = "com.russellbanks"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    // Commons IO - https://commons.apache.org/proper/commons-io/
    implementation(libs.commons.io)

    // Kotlin Coroutines - https://github.com/Kotlin/kotlinx.coroutines
    implementation(libs.coroutines.core)

    // Crypto - https://github.com/appmattus/crypto
    implementation(libs.crypto.cryptohash)

    // Detekt Formatting Plugin - https://github.com/detekt/detekt
    detektPlugins(libs.detekt.formatting)

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

tasks.withType<KotlinCompile> {
    kotlinOptions.jvmTarget = JavaVersion.VERSION_17.toString()
}

application {
    mainClass.set("MainKt")
}