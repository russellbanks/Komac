package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object InstallerScope : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val installerScopeSchema = schemasImpl.installerSchema.definitions.scope
    private val previousManifestData: PreviousManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()

    fun Terminal.installerScopePrompt() {
        when (sharedManifestData.msi?.allUsers) {
            "1" -> installerManifestData.scope = InstallerManifest.Installer.Scope.Machine
            "" -> installerManifestData.scope = InstallerManifest.Installer.Scope.User
            "2" -> installerManifestData.scope = null
            else -> {
                if (
                    installerManifestData.installerType == InstallerManifest.Installer.InstallerType.MSIX ||
                    installerManifestData.installerType == InstallerManifest.Installer.InstallerType.APPX
                ) {
                    installerManifestData.scope = InstallerManifest.Installer.Scope.User
                    return
                }
                do {
                    val previousValue = getPreviousValue()
                    println(
                        verticalLayout {
                            cell(brightYellow(installerScopeInfo))
                            InstallerManifest.Scope.values().forEach { scope ->
                                val textColour = when (previousValue) {
                                    scope, scope.toPerScopeInstallerType() -> brightGreen
                                    else -> brightWhite
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
                            previousValue?.let { cell(gray("Previous value: $previousValue")) }
                        }
                    )
                    val input = prompt(
                        prompt = brightWhite(Prompts.enterChoice),
                        default = previousValue?.toString()?.first()?.toString()
                    )?.trim()
                    val (installerScopeValid, error) = isInstallerScopeValid(input?.firstOrNull(), installerScopeSchema)
                    if (installerScopeValid == Validation.Success) {
                        installerManifestData.scope = InstallerManifest.Installer.Scope.values().firstOrNull {
                            it.name.firstOrNull()?.titlecase() == input?.firstOrNull()?.titlecase()
                        }
                    }
                    error?.let { println(brightRed(it)) }
                    println()
                } while (installerScopeValid != Validation.Success)
            }
        }
    }

    private fun getPreviousValue(): Enum<*>? {
        return previousManifestData.remoteInstallerData?.let {
            it.scope ?: it.installers[installerManifestData.installers.size].scope
        }
    }

    private fun isInstallerScopeValid(
        option: Char?,
        installerScopeSchema: InstallerSchema.Definitions.Scope
    ): Pair<Validation, String?> {
        return when {
            option == null || option.isWhitespace() -> Validation.Success to null
            option != Prompts.noIdea.first() && installerScopeSchema.enum.all {
                it.first().titlecase() != option.titlecase()
            } -> Validation.InvalidInstallerScope to Errors.invalidEnum(
                Validation.InvalidInstallerScope,
                installerScopeSchema.enum
            )
            else -> Validation.Success to null
        }
    }

    private const val installerScopeInfo = "${Prompts.optional} Enter the Installer Scope"
}
