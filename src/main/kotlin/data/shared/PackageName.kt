package data.shared

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.terminal.Terminal
import data.PreviousManifestData
import data.SharedManifestData
import input.PromptType
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.DefaultLocaleSchema

object PackageName : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val packageNameSchema = get<SchemasImpl>().defaultLocaleSchema.properties.packageName

    fun Terminal.packageNamePrompt() {
        sharedManifestData.msix?.displayName?.let {
            sharedManifestData.packageName = it
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
                sharedManifestData.packageName = input
            }
            error?.let { println(brightRed(it)) }
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
