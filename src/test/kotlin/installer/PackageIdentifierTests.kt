package installer
import InstallerSchemaData
import Validation
import data.InstallerManifestChecks
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe

class PackageIdentifierTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Package Identifier Tests") {
        withData(
            listOf(
                "ThisIsATest.Test",
                "Test.test",
                "test.test"
            )
        ) { identifier ->
            InstallerManifestChecks.isPackageIdentifierValid(identifier, installerSchema).first
                .shouldBe(Validation.Success)
        }

        withData(
            listOf(
                null,
                "test",
                ".",
                "test./",
                "test/test",
            )
        ) { identifier ->
            InstallerManifestChecks.isPackageIdentifierValid(identifier, installerSchema).first
                .shouldNotBe(Validation.Success)
        }
    }
})