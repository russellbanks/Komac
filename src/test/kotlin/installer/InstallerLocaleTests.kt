package installer

import InstallerSchemaData
import Validation
import data.shared.Locale.isInstallerLocaleValid
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import java.util.Locale

class InstallerLocaleTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema
    val listSize = 10

    context("Installer Locale Checks") {
        withData(Locale.getISOLanguages().toList().shuffled().take(listSize)) { language ->
            withData(Locale.getISOCountries().toList().shuffled().take(listSize)) { country ->
                isInstallerLocaleValid(
                    locale = "$language-$country",
                    installerSchema = installerSchema
                ).first shouldBe Validation.Success
            }
        }
    }
})
