package data.locale

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object Publisher : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val publisherSchema = get<SchemasImpl>().defaultLocaleSchema.properties.publisher

    fun Terminal.publisherPrompt() {
        do {
            println(brightGreen(publisherInfo))
            println(cyan(publisherExample))
            val input = prompt(
                prompt = brightWhite(PromptType.Publisher.toString()),
                default = previousManifestData.remoteDefaultLocaleData?.publisher?.also {
                    println(gray("Previous publisher: $it"))
                }
            )?.trim()
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

    private val publisherInfo = "${Prompts.required} Enter ${publisherSchema.description.lowercase()}"
    private const val publisherExample = "Example: Microsoft Corporation"
}
