package installer

import InstallerSchemaData
import InstallerSwitch
import Validation
import data.InstallerManifestChecks
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
                InstallerManifestChecks.isInstallerSwitchValid(
                    switch = switchString,
                    installerSwitch = installerSwitch,
                    canBeBlank = false,
                    installerSchema = installerSchema
                ).first.shouldBe(Validation.Success)
            }
        }
    }
})
