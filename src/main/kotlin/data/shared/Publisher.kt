package data.shared

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.PreviousManifestData
import data.SharedManifestData
import input.ExitCode
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.DefaultLocaleManifest
import kotlin.system.exitProcess

object Publisher : KoinComponent, CommandPrompt<String> {
    private val previousManifestData: PreviousManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()

    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        return sharedManifestData.msi?.manufacturer ?: sharedManifestData.msix?.publisherDisplayName ?: let {
            println(colors.brightGreen(publisherInfo))
            info(example)
            prompt(
                prompt = const,
                default = previousManifestData.remoteDefaultLocaleData.await()?.publisher
                    ?.also { muted("Previous publisher: $it") },
                convert = { input ->
                    getError(input)
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input.trim())
                }
            )?.also { println() } ?: exitProcess(ExitCode.CtrlC.code)
        }
    }

    override fun getError(input: String?): String? {
        return when {
            input == null -> null
            input.isBlank() -> Errors.blankInput(const)
            input.length < minLength || input.length > maxLength -> {
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
