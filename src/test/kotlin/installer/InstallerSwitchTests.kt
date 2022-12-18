package installer

import InstallerSchemaData
import Validation
import data.InstallerSwitch.isInstallerSwitchValid
import input.InstallerSwitch
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class InstallerSwitchTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Installer Switch Tests") {
        withData(InstallerSwitch.values().toList()) { installerSwitch ->
            withData(
                listOf(
                    "/S",
                    "-silent",
                    "/silent",
                    "-SILENT",
                    "/norestart",
                    "-norestart"
                )
            ) { switchString ->
                isInstallerSwitchValid(
                    switch = switchString,
                    installerSwitch = installerSwitch,
                    canBeBlank = false,
                    installerSchema = installerSchema
                ).first shouldBe Validation.Success
            }
        }
    }
})
