package installer

import InstallerSchemaData
import Validation
import data.InstallerManifestChecks
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import java.util.Locale

class LocaleChecks : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema
    context("Installer Locale Checks") {
        withData(Locale.getISOLanguages().toList()) { language ->
            withData(Locale.getISOCountries().toList()) { country ->
                InstallerManifestChecks.isInstallerLocaleValid(
                    locale = "$language-$country",
                    installerSchema = installerSchema
                ).first.shouldBe(Validation.Success)
            }
        }
    }
})