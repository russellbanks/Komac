package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.InstallerSchema
import schemas.SchemasImpl

object ProductCode : KoinComponent {
    fun Terminal.productCodePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val schemasImpl: SchemasImpl by inject()
        val productCodeSchema = schemasImpl.installerSchema.definitions.productCode
        do {
            println(brightYellow(productCodeInfo))
            installerManifestData.productCode = prompt(brightWhite(PromptType.ProductCode.toString()))?.trim()
            val (productCodeValid, error) = isProductCodeValid(installerManifestData.productCode, productCodeSchema)
            error?.let { println(red(it)) }
            println()
        } while (productCodeValid != Validation.Success)
    }

    fun isProductCodeValid(
        productCode: String?,
        productCodeSchema: InstallerSchema.Definitions.ProductCode
    ): Pair<Validation, String?> {
        return when {
            !productCode.isNullOrBlank() && productCode.length > productCodeSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = productCodeSchema.minLength,
                    max = productCodeSchema.maxLength
                )
            }
            else -> Validation.Success to null
        }
    }

    private const val productCodeInfo = "${Prompts.optional} Enter the application product code. " +
        "Looks like {CF8E6E00-9C03-4440-81C0-21FACB921A6B}"
}
