package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.PromptType
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.InstallerSchema

object FileExtensions : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val fileExtensionsSchema = get<SchemasImpl>().installerSchema.definitions.fileExtensions

    fun Terminal.fileExtensionsPrompt() {
        do {
            println(
                brightYellow(
                    "${Prompts.optional} ${fileExtensionsSchema.description} (Max ${fileExtensionsSchema.maxItems})"
                )
            )
            val input = prompt(
                prompt = brightWhite(PromptType.FileExtensions.toString()),
                default = getPreviousValue()?.joinToString(", ")?.also {
                    muted("Previous file extensions: $it")
                }
            )?.trim()?.convertToYamlList(fileExtensionsSchema.uniqueItems)
            val (fileExtensionsValid, error) = areFileExtensionsValid(input)
            if (fileExtensionsValid == Validation.Success) installerManifestData.fileExtensions = input
            error?.let { println(brightRed(it)) }
            println()
        } while (fileExtensionsValid != Validation.Success)
    }

    private fun areFileExtensionsValid(
        fileExtensions: Iterable<String>?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val fileExtensionsSchema = installerSchema.definitions.fileExtensions
        return when {
            (fileExtensions?.count() ?: 0) > fileExtensionsSchema.maxItems -> {
                Validation.InvalidLength to Errors.invalidLength(max = fileExtensionsSchema.maxItems)
            }
            fileExtensions?.any { !it.matches(Regex(fileExtensionsSchema.items.pattern)) } == true -> {
                Validation.InvalidPattern to Errors.invalidRegex(
                    regex = Regex(fileExtensionsSchema.items.pattern),
                    items = fileExtensions.filterNot { it.matches(Regex(fileExtensionsSchema.items.pattern)) }
                )
            }
            fileExtensions?.any { it.length > fileExtensionsSchema.items.maxLength } == true -> {
                Validation.InvalidLength to Errors.invalidLength(
                    max = fileExtensionsSchema.items.maxLength,
                    items = fileExtensions.filter { it.length > fileExtensionsSchema.items.maxLength }
                )
            }
            else -> Validation.Success to null
        }
    }

    private fun getPreviousValue(): List<String>? {
        return previousManifestData.remoteInstallerData?.let {
            it.fileExtensions ?: it.installers[installerManifestData.installers.size].fileExtensions
        }
    }
}
