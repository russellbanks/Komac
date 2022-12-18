package installer

import InstallerSchemaData
import Validation
import data.PackageVersion.isPackageVersionValid
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import kotlin.random.Random

class PackageVersionTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Package Version Tests") {
        withData(
            List(50) {
                val major = Random.nextInt(1, 101)
                val minor = Random.nextInt(1, 101)
                val patch = Random.nextInt(1, 1001)
                "${if (it % 2 == 0) "v" else ""}$major.$minor.$patch"
            }
        ) { version ->
            isPackageVersionValid(version, installerSchema).first shouldBe Validation.Success
        }

        withData(
            listOf(
                null,
                "/",
                "?"
            )
        ) { version ->
            isPackageVersionValid(version, installerSchema).first shouldNotBe Validation.Success
        }
    }
})
