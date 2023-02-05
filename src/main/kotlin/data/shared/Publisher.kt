package data.shared

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.PreviousManifestData
import data.SharedManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.DefaultLocaleManifest
import kotlin.system.exitProcess

object Publisher : KoinComponent {
    private val previousManifestData: PreviousManifestData by inject()
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
                        getPublisherError(input)
                            ?.let { ConversionResult.Invalid(it) }
                            ?: ConversionResult.Valid(input.trim())
                    }
                )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
            }
        }
    }

    private fun getPublisherError(publisher: String): String? {
        return when {
            publisher.isBlank() -> Errors.blankInput(const)
            publisher.length < minLength || publisher.length > maxLength -> {
                Errors.invalidLength(min = minLength, max = maxLength)
            }
            else -> null
        }
    }

    private val const = DefaultLocaleManifest::publisher.name.replaceFirstChar { it.titlecase() }
    private const val publisherInfo = "${Prompts.required} Enter the publisher name"
    private const val example = "Example: Microsoft Corporation"
    private const val minLength = 2
    private const val maxLength = 256
}
