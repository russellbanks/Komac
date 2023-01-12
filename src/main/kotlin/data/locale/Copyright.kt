package data.locale

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object Copyright : KoinComponent {
    fun Terminal.copyrightPrompt() {
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        val copyrightSchema = get<SchemasImpl>().defaultLocaleSchema.properties.copyright
        do {
            println(brightYellow(copyrightInfo(copyrightSchema)))
            println(cyan(copyrightExample))
            val input = prompt(brightWhite(PromptType.Copyright.toString()))?.trim()
            val (packageLocaleValid, error) = isCopyrightValid(input, copyrightSchema)
            if (packageLocaleValid == Validation.Success && input != null) {
                defaultLocaleManifestData.copyright = input
            }
            error?.let { println(brightRed(it)) }
            println()
        } while (packageLocaleValid != Validation.Success)
    }

    fun isCopyrightValid(
        copyright: String?,
        copyrightSchema: DefaultLocaleSchema.Properties.Copyright
    ): Pair<Validation, String?> {
        return when {
            !copyright.isNullOrBlank() &&
                (copyright.length < copyrightSchema.minLength || copyright.length > copyrightSchema.maxLength) -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = copyrightSchema.minLength,
                    max = copyrightSchema.maxLength
                )
            }
            else -> Validation.Success to null
        }
    }

    private fun copyrightInfo(copyrightSchema: DefaultLocaleSchema.Properties.Copyright): String {
        return "${Prompts.optional} Enter ${copyrightSchema.description.lowercase()}"
    }

    private const val copyrightExample = "Example: Copyright (c) Microsoft Corporation"
}
