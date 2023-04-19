package utils

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import okio.Path
import okio.Path.Companion.toPath
import okio.fakefilesystem.FakeFileSystem
import schemas.manifest.InstallerManifest.Installer.Architecture
import schemas.manifest.InstallerManifest.InstallerType

class FileAnalyserTests : FunSpec({
    val fileSystem = FakeFileSystem()
    val directory = "/Users/user".toPath()

    beforeEach {
        fileSystem.createDirectories(directory)
    }

    afterEach {
        fileSystem.checkNoOpenFiles()
        fileSystem.deleteRecursively(directory)
    }

    context("inno detection") {
        test("exe contains expected inno bytes") {
            val file = directory / "innoFile.exe"
            fileSystem.write(file) { write(FileAnalyser.innoBytes) }
            FileAnalyser(file, fileSystem).installerType shouldBe InstallerType.INNO
        }

        test("exe contains incorrect amount of inno bytes") {
            val file = directory / "innoFile.exe"
            fileSystem.write(file) {
                write(FileAnalyser.innoBytes.substring(0, FileAnalyser.innoBytes.size - 2))
                repeat(8) { writeByte(0) }
            }
            FileAnalyser(file, fileSystem).installerType shouldBe null
        }
    }

    context("nullsoft detection") {
        test("exe contains expected nullsoft bytes") {
            val file = directory / "nullsoft.exe"
            fileSystem.write(file) { write(FileAnalyser.nullsoftBytes) }
            FileAnalyser(file, fileSystem).installerType shouldBe InstallerType.NULLSOFT
        }

        test("exe contains incorrect amount of nullsoft bytes") {
            val file = directory / "innoFile.exe"
            fileSystem.write(file) {
                write(FileAnalyser.nullsoftBytes.substring(0, FileAnalyser.nullsoftBytes.size - 2))
                repeat(8) { writeByte(0) }
            }
            FileAnalyser(file, fileSystem).installerType shouldBe null
        }
    }

    context("burn detection") {
        test("exe contains burn header") {
            val file = directory / "burn.exe"
            fileSystem.write(file) {
                repeat(64) { writeByte(0) }
                writeString(FileAnalyser.wixBurnHeader, Charsets.UTF_8)
            }
            FileAnalyser(file, fileSystem).installerType shouldBe InstallerType.BURN
        }

        test("exe contains burn header in wrong place") {
            val file = directory / "burn.exe"
            fileSystem.write(file) {
                repeat(UShort.MAX_VALUE.toInt() * FileAnalyser.burnBufferSize.toInt()) { writeByte(0) }
                writeString(FileAnalyser.wixBurnHeader, Charsets.UTF_8)
            }
            FileAnalyser(file, fileSystem).installerType shouldBe null
        }
    }

    context("get architecture") {
        fun Path.writeExeWithMachine(machine: Int) {
            val peHeaderSize = 0x108
            fileSystem.write(this) {
                writeUtf8("MZ")
                repeat(FileAnalyser.peHeaderLocation.toInt() - 2) { writeByte(0) }
                writeIntLe(peHeaderSize)
                repeat(peHeaderSize - FileAnalyser.peHeaderLocation.toInt()) { writeByte(0) }
                writeShortLe(machine)
            }
        }

        test("should return 8664 when machine value is 0x8664") {
            val file = directory / "64bit.exe"
            file.writeExeWithMachine(0x8664)
            FileAnalyser(file, fileSystem).peArchitectureValue shouldBe "8664"
        }

        test("should return x64 when machine value is 0x8664") {
            val file = directory / "64bit.exe"
            file.writeExeWithMachine(0x8664)
            FileAnalyser(file, fileSystem).architecture shouldBe Architecture.X64
        }

        test("should return x86 when machine value is 0x8664") {
            val file = directory / "32bit.exe"
            file.writeExeWithMachine(0x014c)
            FileAnalyser(file, fileSystem).architecture shouldBe Architecture.X86
        }

        test("should return 14c when machine value is 0x014c") {
            val file = directory / "32bit.exe"
            file.writeExeWithMachine(0x014c)
            FileAnalyser(file, fileSystem).peArchitectureValue shouldBe "14c"
        }
    }
})
