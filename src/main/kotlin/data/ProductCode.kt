package data

import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerSchema
import schemas.InstallerSchemaImpl

object ProductCode : KoinComponent {
    fun Terminal.productCodePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        do {
            println(brightYellow(Prompts.productCodeInfo))
            installerManifestData.productCode = prompt(brightWhite(PromptType.ProductCode.toString()))?.trim()
            val (productCodeValid, error) = isProductCodeValid(installerManifestData.productCode)
            error?.let { println(red(it)) }
            println()
        } while (productCodeValid != Validation.Success)
    }

    fun isProductCodeValid(
        productCode: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val productCodeMinLength = installerSchema.definitions.productCode.minLength
        val productCodeMaxLength = installerSchema.definitions.productCode.maxLength
        return when {
            !productCode.isNullOrBlank() && productCode.length > productCodeMaxLength -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = productCodeMinLength,
                    max = productCodeMaxLength
                )
            }
            else -> Validation.Success to null
        }
    }
}
