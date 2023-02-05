package data.locale

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.DefaultLocaleManifest
import kotlin.system.exitProcess

object Moniker : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.monikerPrompt() {
        println(colors.brightYellow(monikerInfo))
        info(monikerExample)
        defaultLocaleManifestData.moniker = prompt(
            prompt = DefaultLocaleManifest::moniker.name.replaceFirstChar { it.titlecase() },
            default = previousManifestData.remoteDefaultLocaleData?.moniker?.also { muted("Previous moniker: $it") },
            convert = { input ->
                isMonikerValid(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun isMonikerValid(moniker: String): String? {
        return when {
            moniker.isNotBlank() && (moniker.length < minLength || moniker.length > maxLength) -> {
                Errors.invalidLength(min = minLength, max = maxLength)
            }
            else -> null
        }
    }

    private const val monikerInfo = "${Prompts.optional} Enter the Moniker (friendly name/alias)."
    private const val monikerExample = "Example: vscode"
    private const val minLength = 1
    private const val maxLength = 40
}
