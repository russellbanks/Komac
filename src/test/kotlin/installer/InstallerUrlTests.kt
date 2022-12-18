package installer

import InstallerSchemaData
import Validation
import data.InstallerUrl.isInstallerUrlValid
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class InstallerUrlTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema
    context("Installer Url Tests") {
        withData(
            listOf("https://github.com")
        ) { url ->
            isInstallerUrlValid(url, installerSchema).first shouldBe Validation.Success
        }
    }
})
