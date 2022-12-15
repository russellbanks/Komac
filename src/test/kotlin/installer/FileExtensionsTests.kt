package installer

import InstallerSchemaData
import Validation
import data.InstallerManifestChecks
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
            InstallerManifestChecks.areFileExtensionsValid(it, installerSchema).first.shouldBe(Validation.Success)
        }
    }
})
