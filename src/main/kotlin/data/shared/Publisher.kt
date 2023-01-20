package data.shared

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.terminal.Terminal
import data.PreviousManifestData
import data.SharedManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.DefaultLocaleSchema

object Publisher : KoinComponent {
    private val previousManifestData: PreviousManifestData by inject()
    private val publisherSchema = get<SchemasImpl>().defaultLocaleSchema.properties.publisher
    private val sharedManifestData: SharedManifestData by inject()

    fun Terminal.publisherPrompt() {
        sharedManifestData.msix?.publisherDisplayName?.let {
            sharedManifestData.publisher = it
            return
        }
        sharedManifestData.msi?.manufacturer?.let {
            sharedManifestData.publisher = it
            return
        }
        do {
            println(colors.brightGreen(publisherInfo))
            info(publisherExample)
            val input = prompt(
                prompt = brightWhite(PromptType.Publisher.toString()),
                default = previousManifestData.remoteDefaultLocaleData?.publisher?.also {
                    muted("Previous publisher: $it")
                }
            )?.trim()
            val (publisherValid, error) = publisherValid(input, publisherSchema)
            if (publisherValid == Validation.Success && input != null) {
                sharedManifestData.publisher = input
            }
            error?.let { println(brightRed(it)) }
            println()
        } while (publisherValid != Validation.Success)
    }

    private fun publisherValid(
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
