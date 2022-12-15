package installer

import InstallerSchemaData
import Validation
import data.InstallerManifestChecks
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import schemas.Enum

class ArchitectureTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Architecture Tests") {
        withData(Enum.architecture(installerSchema)) {
            InstallerManifestChecks.isArchitectureValid(it, installerSchema).first.shouldBe(Validation.Success)
        }

        withData(
            listOf("64", "86", "x32", "64bit", "32bit", "arm32", "arm32bit", "arm64bit", "x64bit", null)
        ) {
            InstallerManifestChecks.isArchitectureValid(it, installerSchema).first.shouldNotBe(Validation.Success)
        }
    }
})
