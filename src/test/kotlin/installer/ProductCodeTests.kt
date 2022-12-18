package installer

import InstallerSchemaData
import Validation
import data.ProductCode.isProductCodeValid
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import java.util.UUID

class ProductCodeTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Product Code Checks") {
        withData(
            List(50) {
                "{${UUID.randomUUID().toString().uppercase()}}"
            }
        ) {
            isProductCodeValid(it, installerSchema).first shouldBe Validation.Success
        }
    }
})
