package installer

import InstallerSchemaData
import Validation
import data.installer.InstallerType.isInstallerTypeValid
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe

class InstallerTypeTests : FunSpec({
    val installerTypeSchema = InstallerSchemaData.installerSchema.definitions.installerType

    context("Installer Type Checks") {
        withData(installerTypeSchema.enum) {
            isInstallerTypeValid(it, installerTypeSchema).first shouldBe Validation.Success
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
            isInstallerTypeValid(it, installerTypeSchema).first shouldNotBe Validation.Success
        }
    }
})
