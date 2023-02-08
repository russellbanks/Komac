package data.installer

import Errors
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.ExitCode
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest
import kotlin.system.exitProcess

object InstallerScope : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    suspend fun Terminal.installerScopePrompt() {
        if (
            installerManifestData.scope == null &&
            installerManifestData.installerType != InstallerManifest.Installer.InstallerType.PORTABLE
        ) {
            val previousValue = getPreviousValue()
            println(
                verticalLayout {
                    cell(colors.brightYellow(installerScopeInfo))
                    InstallerManifest.Scope.values().forEach { scope ->
                        val textColour = when (previousValue) {
                            scope, scope.toPerScopeInstallerType() -> colors.brightGreen
                            else -> colors.brightWhite
                        }
                        cell(
                            textColour(
                                buildString {
                                    append(" ".repeat(Prompts.optionIndent))
                                    append("[${scope.toString().first().titlecase()}] ")
                                    append(scope.toString().replaceFirstChar { it.titlecase() })
                                }
                            )
                        )
                    }
                    previousValue?.let { cell(colors.muted("Previous value: $previousValue")) }
                }
            )
            val input = prompt(
                prompt = Prompts.enterChoice,
                default = previousValue?.toString()?.first()?.toString(),
                convert = { input ->
                    getInstallerScopeError(input.trim())
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input.trim())
                }
            ) ?: exitProcess(ExitCode.CtrlC.code)
            installerManifestData.scope = InstallerManifest.Installer.Scope.values().firstOrNull {
                it.name.firstOrNull()?.titlecase() == input.firstOrNull()?.titlecase()
            }
            println()
        }
    }

    private suspend fun getPreviousValue(): Enum<*>? {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.scope ?: it.installers.getOrNull(installerManifestData.installers.size)?.scope
        }
    }

    private fun getInstallerScopeError(option: String): String? {
        val installerScopeValues = InstallerManifest.Scope.values().map { it.toString() }
        return when {
            option.isBlank() -> null
            option.firstOrNull()?.lowercaseChar() !in installerScopeValues.map { it.first().lowercaseChar() } -> {
                Errors.invalidEnum(installerScopeValues)
            }
            else -> null
        }
    }

    const val const = "Installer Scope"
    private const val installerScopeInfo = "${Prompts.optional} Enter the $const"
}
