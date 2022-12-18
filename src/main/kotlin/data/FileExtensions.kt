package data

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
import schemas.InstallerSchemaImpl
import schemas.Pattern

object FileExtensions : KoinComponent {
    fun Terminal.fileExtensionsPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val installerSchemaImpl: InstallerSchemaImpl = get()
        val uniqueItems = installerSchemaImpl.installerSchema.definitions.fileExtensions.uniqueItems
        do {
            println(brightYellow(Prompts.fileExtensionsInfo(installerSchemaImpl.installerSchema)))
            val input = prompt(brightWhite(PromptType.FileExtensions.toString()))
                ?.trim()?.convertToYamlList(uniqueItems)
            val (fileExtensionsValid, error) = areFileExtensionsValid(input)
            if (fileExtensionsValid == Validation.Success) installerManifestData.fileExtensions = input
            error?.let { println(red(it)) }
            println()
        } while (fileExtensionsValid != Validation.Success)
    }

    fun areFileExtensionsValid(
        fileExtensions: Iterable<String>?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val fileExtensionsMaxItems = installerSchema.definitions.fileExtensions.maxItems
        return when {
            (fileExtensions?.count() ?: 0) > fileExtensionsMaxItems -> {
                Validation.InvalidLength to Errors.invalidLength(max = fileExtensionsMaxItems)
            }
            fileExtensions?.any { !it.matches(Pattern.fileExtension(installerSchema)) } == true -> {
                Validation.InvalidPattern to Errors.invalidRegex(
                    Pattern.fileExtension(installerSchema),
                    mutableListOf<String>().apply {
                        fileExtensions.forEach {
                            if (!it.matches(Pattern.fileExtension(installerSchema))) add(it)
                        }
                    }
                )
            }
            else -> Validation.Success to null
        }
    }
}
