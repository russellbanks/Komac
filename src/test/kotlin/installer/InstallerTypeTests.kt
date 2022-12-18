package installer

import InstallerSchemaData
import Validation
import data.InstallerType.isInstallerTypeValid
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import schemas.Enum

class InstallerTypeTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Installer Type Checks") {
        withData(Enum.installerType(installerSchema)) {
            isInstallerTypeValid(it, installerSchema).first shouldBe Validation.Success
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
            isInstallerTypeValid(it, installerSchema).first shouldNotBe Validation.Success
        }
    }
})
