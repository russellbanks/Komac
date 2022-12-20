package installer

import InstallerSchemaData
import Validation
import data.Url.isUrlValid
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class InstallerUrlTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema
    context("Installer Url Tests") {
        withData(
            listOf("https://github.com")
        ) { url ->
            isUrlValid(url, installerSchema).first shouldBe Validation.Success
        }
    }
})
