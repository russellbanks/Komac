package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.PreviousManifestData
import data.SharedManifestData
import input.ExitCode
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.manifest.DefaultLocaleManifest
import kotlin.system.exitProcess

object License : KoinComponent, CommandPrompt<String>  {
    private val previousManifestData: PreviousManifestData by inject()

    override suspend fun prompt(terminal: Terminal): String = with (terminal) {
        return get<SharedManifestData>().gitHubDetection?.license?.await() ?: let {
            println(colors.brightGreen(licenseInfo))
            info(example)
            prompt(
                prompt = const,
                default = previousManifestData.remoteDefaultLocaleData.await()?.license
                    ?.also { muted("Previous license: $it") },
                convert = { input ->
                    getError(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
                }
            ).also { println() } ?: exitProcess(ExitCode.CtrlC.code)
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

    private val const = DefaultLocaleManifest::license.name.replaceFirstChar { it.titlecase() }
    private const val licenseInfo = "${Prompts.required} Enter the package license"
    private const val example = "Example: MIT, GPL-3.0, Freeware, Proprietary"
    private const val minLength = 3
    private const val maxLength = 512
}
