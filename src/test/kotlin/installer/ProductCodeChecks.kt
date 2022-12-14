package installer

import InstallerSchemaData
import Validation
import data.InstallerManifestChecks
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import java.util.UUID

class ProductCodeChecks : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Product Code Checks") {
        withData(
            List(50) {
                "{${UUID.randomUUID().toString().uppercase()}}"
            }
        ) {
            InstallerManifestChecks.isProductCodeValid(it, installerSchema).first.shouldBe(Validation.Success)
        }
    }
})