package data.shared

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.PreviousManifestData
import data.SharedManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import kotlin.system.exitProcess

object Publisher : KoinComponent {
    private val previousManifestData: PreviousManifestData by inject()
    private val publisherSchema = get<SchemasImpl>().defaultLocaleSchema.properties.publisher
    private val sharedManifestData: SharedManifestData by inject()

    fun Terminal.publisherPrompt() {
        sharedManifestData.publisher = when {
            sharedManifestData.msix?.publisherDisplayName != null -> sharedManifestData.msix?.publisherDisplayName
            sharedManifestData.msi?.manufacturer != null -> sharedManifestData.msi?.manufacturer
            else -> {
                println(colors.brightGreen(publisherInfo))
                info(example)
                prompt(
                    prompt = const,
                    default = previousManifestData.remoteDefaultLocaleData?.publisher
                        ?.also { muted("Previous publisher: $it") },
                    convert = { input ->
                        publisherValid(input)
                            ?.let { ConversionResult.Invalid(it) }
                            ?: ConversionResult.Valid(input.trim())
                    }
                )?.also { println(it) } ?: exitProcess(ExitCode.CtrlC.code)
            }
        }
    }

    private fun publisherValid(publisher: String): String? {
        return when {
            publisher.isBlank() -> Errors.blankInput(const)
            publisher.length < publisherSchema.minLength || publisher.length > publisherSchema.maxLength -> {
                Errors.invalidLength(min = publisherSchema.minLength, max = publisherSchema.maxLength)
            }
            else -> null
        }
    }

    private const val const = "Publisher"
    private val publisherInfo = "${Prompts.required} Enter ${publisherSchema.description.lowercase()}"
    private const val example = "Example: Microsoft Corporation"
}
