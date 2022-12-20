package installer

import InstallerSchemaData
import Validation
import data.installer.Protocols
import input.YamlExtensions.convertToYamlList
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class ProtocolTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Protocol Tests") {
        withData(
            listOf(
                listOf("http", "https"),
                listOf("ftp", "sftp")
            )
        ) {
            Protocols.areProtocolsValid(it, installerSchema).first shouldBe Validation.Success
        }

        test("Test protocols sorting with duplicates, spaces, and an empty string") {
            val input = "http, https, ftp, sftp, http, https, , "
            val expected = listOf("ftp", "http", "https", "sftp")
            input.convertToYamlList() shouldBe expected
        }

        test("Test protocol max length") {
            val protocolMaxLength = installerSchema.definitions.protocols.items.maxLength
            val input = listOf("a".repeat(protocolMaxLength + 1))
            Protocols.areProtocolsValid(input, installerSchema).first shouldBe Validation.InvalidLength
        }
    }
})
