package installer

import InstallerSchemaData
import Validation
import data.InstallerManifestChecks
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import schemas.Enum

class InstallerTypeTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Installer Type Checks") {
        withData(Enum.installerType(installerSchema)) {
            InstallerManifestChecks.isInstallerTypeValid(it, installerSchema).first.shouldBe(Validation.Success)
        }

        withData(
            listOf(
                "msixx",
                "appxx",
                "exx",
                "zipp",
                "inn",
                "nullsof",
                "wixx",
                "burnn",
                "pwaa",
                "portablee",
                null
            )
        ) {
            InstallerManifestChecks.isInstallerTypeValid(it, installerSchema).first.shouldNotBe(Validation.Success)
        }
    }
})
