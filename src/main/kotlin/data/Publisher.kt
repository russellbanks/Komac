package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object Publisher : KoinComponent {
    fun Terminal.publisherPrompt() {
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        val schemasImpl: SchemasImpl by inject()
        val publisherSchema = schemasImpl.defaultLocaleSchema.properties.publisher
        do {
            println(brightGreen(publisherInfo(publisherSchema)))
            val input = prompt(brightWhite(PromptType.Publisher.toString()))?.trim()
            val (publisherValid, error) = publisherValid(input, publisherSchema)
            if (publisherValid == Validation.Success && input != null) {
                defaultLocaleManifestData.publisher = input
            }
            error?.let { println(red(it)) }
            println()
        } while (publisherValid != Validation.Success)
    }

    fun publisherValid(
        publisher: String?,
        publisherSchema: DefaultLocaleSchema.Properties.Publisher
    ): Pair<Validation, String?> {
        return when {
            publisher.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.Publisher)
            publisher.length < publisherSchema.minLength || publisher.length > publisherSchema.maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = publisherSchema.minLength,
                    max = publisherSchema.maxLength
                )
            }
            else -> Validation.Success to null
        }
    }

    private fun publisherInfo(publisherSchema: DefaultLocaleSchema.Properties.Publisher): String {
        return "Enter ${publisherSchema.description.lowercase()}. For example: Microsoft Corporation"
    }
}
