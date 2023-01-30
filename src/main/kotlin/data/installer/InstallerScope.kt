package data.installer

import Errors
import ExitCode
import Validation
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.manifest.InstallerManifest
import kotlin.system.exitProcess

object InstallerScope : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val installerScopeSchema = schemasImpl.installerSchema.definitions.scope
    private val previousManifestData: PreviousManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()

    fun Terminal.installerScopePrompt() {
        if (sharedManifestData.msi?.allUsers == null) {
            when (installerManifestData.installerType) {
                InstallerManifest.Installer.InstallerType.MSIX, InstallerManifest.Installer.InstallerType.APPX -> {
                    installerManifestData.scope = InstallerManifest.Installer.Scope.User
                }
                InstallerManifest.Installer.InstallerType.PORTABLE -> Unit
                else -> {
                    val previousValue = getPreviousValue()
                    println(
                        verticalLayout {
                            cell(TextColors.brightYellow(installerScopeInfo))
                            InstallerManifest.Scope.values().forEach { scope ->
                                val textColour = when (previousValue) {
                                    scope, scope.toPerScopeInstallerType() -> TextColors.brightGreen
                                    else -> TextColors.brightWhite
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
                            isInstallerScopeValid(input.firstOrNull())
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
        }
    }

    private fun getPreviousValue(): Enum<*>? {
        return previousManifestData.remoteInstallerData?.let {
            it.scope ?: it.installers.getOrNull(installerManifestData.installers.size)?.scope
        }
    }

    private fun isInstallerScopeValid(option: Char?): String? {
        return when {
            option == null || option.isWhitespace() -> null
            option != Prompts.noIdea.first() && installerScopeSchema.enum.all {
                it.first().titlecase() != option.titlecase()
            } -> Errors.invalidEnum(Validation.InvalidInstallerScope, installerScopeSchema.enum)
            else -> null
        }
    }

    const val const = "Installer Scope"

    private const val installerScopeInfo = "${Prompts.optional} Enter the $const"
}
