package installer

import InstallerSchemaData
import Validation
import data.InstallerManifestChecks
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class UrlTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema
    context("Installer Url Tests") {
        withData(
            listOf("https://github.com")
        ) { url ->
            InstallerManifestChecks.isInstallerUrlValid(url, installerSchema).first.shouldBe(Validation.Success)
        }
    }
})