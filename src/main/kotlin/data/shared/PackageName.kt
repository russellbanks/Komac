package data.shared

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.PreviousManifestData
import data.SharedManifestData
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import kotlin.system.exitProcess

object PackageName : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.packageNamePrompt() {
        sharedManifestData.msix?.displayName?.let {
            sharedManifestData.packageName = it
            return
        }
        println(colors.brightGreen(nameInfo))
        info(example)
        sharedManifestData.msi?.productName?.let { info("Detected from MSI: $it") }
        sharedManifestData.packageName = prompt(
            prompt = const,
            default = previousManifestData.remoteDefaultLocaleData?.packageName
                ?.also { muted("Previous package name: $it") },
            convert = { input ->
                isPackageNameValid(input)?.let { ConversionResult.Invalid(it) } ?: ConversionResult.Valid(input.trim())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun isPackageNameValid(input: String): String? {
        return when {
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
