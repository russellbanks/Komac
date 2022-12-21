package installer

import InstallerSchemaData
import Validation
import data.shared.PackageVersion
import data.shared.PackageVersion.isPackageVersionValid
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

    context("Highest version tests") {
        with(PackageVersion) {
            getHighestVersion(listOf("1.0.1", "1.0.0")) shouldBe "1.0.1"
            getHighestVersion(listOf("2", "1")) shouldBe "2"
            getHighestVersion(listOf("14.0", "13.9.8")) shouldBe "14.0"
            getHighestVersion(listOf("14.0", "13.9.8", "14.0.0.1")) shouldBe "14.0.0.1"
            getHighestVersion(listOf("0.0.1-beta", "0.0.1-alpha")) shouldBe "0.0.1-beta"
            getHighestVersion(listOf("0.0.1-beta", "0.0.1-alpha", "0.0.1")) shouldBe "0.0.1"
            getHighestVersion(listOf("0.0.2-alpha", "0.0.1-alpha")) shouldBe "0.0.2-alpha"
            getHighestVersion(listOf("0.0.1-alpha", "0.0.1-alpha01")) shouldBe "0.0.1-alpha01"
            getHighestVersion(listOf("0.0.1-alpha01", "0.0.1-alpha02")) shouldBe "0.0.1-alpha02"
            getHighestVersion(listOf("v1.10.0", "v1.9.0")) shouldBe "v1.10.0"
            getHighestVersion(listOf("v1.10.0", "v1.9.0", "1.10.0.1")) shouldBe "1.10.0.1"
        }
    }
})
