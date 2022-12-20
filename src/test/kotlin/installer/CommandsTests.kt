package installer

import InstallerSchemaData
import Validation
import data.installer.Commands
import input.YamlExtensions.convertToYamlList
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class CommandsTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Command Tests") {
        withData(
            listOf(
                listOf("ls", "cd"),
                listOf("mkdir", "rm")
            )
        ) {
            Commands.areCommandsValid(it, installerSchema).first shouldBe Validation.Success
        }

        test("Test commands sorting with duplicates, spaces, and an empty string") {
            val input = "ls, cd, mkdir, rm, ls, cd, , "
            val expected = listOf("cd", "ls", "mkdir", "rm")
            input.convertToYamlList() shouldBe expected
        }

        test("Test command max length") {
            val commandMaxLength = installerSchema.definitions.commands.items.maxLength
            val input = listOf("a".repeat(commandMaxLength + 1))
            Commands.areCommandsValid(input, installerSchema).first shouldBe Validation.InvalidLength
        }
    }
})
