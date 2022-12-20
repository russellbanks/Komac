package installer

import InstallerSchemaData
import Validation
import data.installer.FileExtensions.areFileExtensionsValid
import input.YamlExtensions.convertToYamlList
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class FileExtensionsTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("File Extension Tests") {
        withData(
            listOf(
                listOf("html", "htm", "docx"),
                listOf("doc", "pdf")
            )
        ) {
            areFileExtensionsValid(it, installerSchema).first shouldBe Validation.Success
        }

        test("Test file extensions sorting with duplicates, spaces, and an empty string") {
            val input = "html, htm, docx, doc, pdf, docx, doc, , "
            val expected = listOf("doc", "docx", "htm", "html", "pdf")
            input.convertToYamlList() shouldBe expected
        }
    }
})
