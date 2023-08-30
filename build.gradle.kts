import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.dsl.KotlinVersion
import org.jetbrains.kotlin.gradle.plugin.mpp.pm20.util.distsDirectory
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile
import org.panteleyev.jpackage.ImageType

plugins {
    alias(libs.plugins.buildconfig)
    alias(libs.plugins.detekt)
    alias(libs.plugins.jpackage)
    alias(libs.plugins.kotest.multiplatform)
    alias(libs.plugins.kotlin)
    alias(libs.plugins.kotlin.serialization)
    alias(libs.plugins.shadow)
    application
}

group = "com.russellbanks"
version = "1.11.0"

repositories {
    mavenCentral()
    maven("https://oss.sonatype.org/content/repositories/snapshots")
}

kotlin {
    jvm {
        withJava()
    }
    mingwX64()
    linuxX64()
    // macosX64()

    sourceSets {
        val commonMain by getting {
            dependencies {
                // Clikt - https://github.com/ajalt/clikt
                implementation(libs.clikt)

                // KMath - https://github.com/SciProgCentre/kmath
                implementation(libs.kmath.core)

                // Kotlin Coroutines - https://github.com/Kotlin/kotlinx.coroutines
                implementation(libs.coroutines.core)

                // KotlinX DateTime - https://github.com/Kotlin/kotlinx-datetime
                implementation(libs.kotlinx.datetime)

                // KotlinX Serialization - https://github.com/Kotlin/kotlinx.serialization
                implementation(libs.kotlinx.serialization.json)

                // Ktor - https://github.com/ktorio/ktor
                implementation(libs.ktor.client.core)

                // Mordant - https://github.com/ajalt/mordant
                implementation(libs.mordant)

                // Okio - https://github.com/square/okio
                implementation(libs.okio)
                implementation(libs.okio.fakefilesystem)
            }
        }

        val commonTest by getting {
            dependencies {
                // Kotest - https://github.com/kotest/kotest
                implementation(libs.kotest.assertions.core)
                implementation(libs.kotest.framework.datatest)
                implementation(libs.kotest.extensions.assertions.ktor)

                // Ktor Mock Engine - https://ktor.io/docs/http-client-testing.html
                implementation(libs.ktor.client.mock)
            }
        }

        val jvmMain by getting {
            dependencies {
                // GitHub API - https://github.com/hub4j/github-api
                implementation(libs.github.api)

                // JLine - https://github.com/jline/jline3
                implementation(libs.jline.terminal.jna)

                // JNA - https://github.com/java-native-access/jna
                implementation(libs.jna)
                implementation(libs.jna.platform)

                // Kaml - https://github.com/charleskorn/kaml
                implementation(libs.kaml)

                // Ktor Java Engine - https://ktor.io/docs/http-client-engines.html#java
                implementation(libs.ktor.client.java)

                // Skrape{it} - https://github.com/skrapeit/skrape.it
                implementation(libs.skrapeit.htmlparser)

                // SLF4J No-operation implementation - https://www.slf4j.org
                implementation(libs.slf4j.nop)
            }
        }

        val jvmTest by getting {
            dependencies {
                // Kotest - https://github.com/kotest/kotest
                implementation(libs.kotest.junit5)

                // Mockk - https://github.com/mockk/mockk
                implementation(libs.mockk)
            }
        }

        val nativeMain by creating {
            dependsOn(commonMain)
        }

        val mingwX64Main by getting {
            dependsOn(nativeMain)

            dependencies {
                // Ktor WinHttp Engine - https://ktor.io/docs/http-client-engines.html#winhttp
                implementation(libs.ktor.client.winhttp)
            }
        }

        val linuxX64Main by getting {
            dependsOn(nativeMain)

            dependencies {
                // Ktor Curl Engine - https://ktor.io/docs/http-client-engines.html#curl
                implementation(libs.ktor.client.curl)
            }
        }

        /*val macosX64Main by getting {
            dependsOn(nativeMain)

            dependencies {
                // Ktor Darwin Engine - https://ktor.io/docs/http-client-engines.html#darwin
                implementation(libs.ktor.client.darwin)
            }
        }*/
    }
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

dependencies {
    // Detekt Formatting Plugin - https://github.com/detekt/detekt
    detektPlugins(libs.detekt.formatting)
}

detekt {
    ignoreFailures = true
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

buildConfig {
    buildConfigField("String", "appName", "\"${project.name}\"")
    buildConfigField("String", "appVersion", "\"${project.version}\"")
    buildConfigField("String", "projectUrl", "\"https://github.com/russellbanks/Komac\"")
}

tasks.withType<KotlinCompile> {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
        languageVersion.set(KotlinVersion.KOTLIN_2_0)
    }
}

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}
