package data.shared

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.AllManifestData
import data.PreviousManifestData
import input.ExitCode
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import kotlin.system.exitProcess

object PackageName : KoinComponent, CommandPrompt<String> {
    private val allManifestData: AllManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    override suspend fun prompt(terminal: Terminal): String = with(terminal) {
        return allManifestData.msix?.displayName ?: let {
            println(colors.brightGreen(nameInfo))
            info(example)
            allManifestData.msi?.productName?.let { info("Detected from MSI: $it") }
            prompt(
                prompt = const,
                default = previousManifestData.remoteDefaultLocaleData.await()?.packageName
                    ?.also { muted("Previous package name: $it") },
                convert = { input ->
                    getError(input.trim())?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
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

    private const val const = "Package Name"
    private const val nameInfo = "Enter the package name"
    private const val example = "Example: Microsoft Teams"
    private const val minLength = 2
    private const val maxLength = 256
}
