package data.shared

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.PromptType
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object PackageName : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val packageNameSchema = get<SchemasImpl>().defaultLocaleSchema.properties.packageName
    private val sharedManifestData: SharedManifestData by inject()

    fun Terminal.packageNamePrompt() {
        sharedManifestData.msix?.displayName?.let {
            defaultLocaleManifestData.packageName = it
            return
        }
        do {
            println(brightGreen(packageNameInfo))
            println(cyan(packageNameExample))
            sharedManifestData.msi?.productName?.let { println(brightYellow("Detected from MSI: $it")) }
            val input = prompt(
                prompt = brightWhite(PromptType.PackageName.toString()),
                default = previousManifestData.remoteDefaultLocaleData?.packageName?.also {
                    println(gray("Previous package name: $it"))
                }
            )?.trim()
            val (packageNameValid, error) = packageNameValid(input, packageNameSchema)
            if (packageNameValid == Validation.Success && input != null) {
                defaultLocaleManifestData.packageName = input
            }
            error?.let { println(red(it)) }
            println()
        } while (packageNameValid != Validation.Success)
    }

    fun packageNameValid(
        input: String?,
        packageNameSchema: DefaultLocaleSchema.Properties.PackageName
    ): Pair<Validation, String?> {
        return when {
            input.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.Publisher)
            input.length < packageNameSchema.minLength || input.length > packageNameSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = packageNameSchema.minLength,
                    max = packageNameSchema.maxLength
                )
            }
            else -> Validation.Success to null
        }
    }

    private val packageNameInfo = "Enter ${packageNameSchema.description.lowercase()}"
    private const val packageNameExample = "For example, Microsoft Teams"
}
