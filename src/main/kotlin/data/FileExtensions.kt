package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerSchema
import schemas.SchemasImpl

object FileExtensions : KoinComponent {
    fun Terminal.fileExtensionsPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val schemasImpl: SchemasImpl by inject()
        val fileExtensionsSchema = schemasImpl.installerSchema.definitions.fileExtensions
        do {
            println(
                brightYellow(
                    "${Prompts.optional} ${fileExtensionsSchema.description} (Max ${fileExtensionsSchema.maxItems})"
                )
            )
            val input = prompt(brightWhite(PromptType.FileExtensions.toString()))
                ?.trim()?.convertToYamlList(fileExtensionsSchema.uniqueItems)
            val (fileExtensionsValid, error) = areFileExtensionsValid(input)
            if (fileExtensionsValid == Validation.Success) installerManifestData.fileExtensions = input
            error?.let { println(red(it)) }
            println()
        } while (fileExtensionsValid != Validation.Success)
    }

    fun areFileExtensionsValid(
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
}
