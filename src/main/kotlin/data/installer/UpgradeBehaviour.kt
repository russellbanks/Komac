package data.installer

import Errors
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.InstallerManifest

object UpgradeBehaviour : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    suspend fun Terminal.upgradeBehaviourPrompt() {
        if (installerManifestData.installerType == InstallerManifest.Installer.InstallerType.PORTABLE) return
        val previousValue = getPreviousValue()
        println(
            verticalLayout {
                cell(colors.brightYellow(upgradeBehaviourInfo))
                InstallerManifest.UpgradeBehavior.values().forEach { behaviour ->
                    val textColour = when {
                        previousValue == behaviour ||
                        previousValue == behaviour.toPerInstallerUpgradeBehaviour() -> colors.brightGreen
                        behaviour == InstallerManifest.UpgradeBehavior.Install -> colors.brightGreen
                        else -> colors.brightWhite
                    }
                    cell(
                        textColour(
                            buildString {
                                append(" ".repeat(Prompts.optionIndent))
                                append("[${behaviour.toString().first().titlecase()}] ")
                                append(behaviour.toString().replaceFirstChar { it.titlecase() })
                            }
                        )
                    )
                }
                previousValue?.let { cell(colors.muted("Previous upgrade behaviour: $previousValue")) }
            }
        )
        val input = prompt(
            prompt = Prompts.enterChoice,
            default = previousValue?.toString()?.firstOrNull()?.toString()
                ?: InstallerManifest.UpgradeBehavior.Install.toString().first().toString(),
            convert = { input ->
                getUpgradeBehaviourError(input.trim())
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )
        installerManifestData.upgradeBehavior = InstallerManifest.Installer.UpgradeBehavior.values().firstOrNull {
            it.name.firstOrNull()?.titlecase() == input?.firstOrNull()?.titlecase()
        }
        println()
    }

    private suspend fun getPreviousValue(): Enum<*>? {
        return previousManifestData.remoteInstallerData.await()?.let {
            it.upgradeBehavior ?: it.installers.getOrNull(installerManifestData.installers.size)?.upgradeBehavior
        }
    }

    private fun getUpgradeBehaviourError(option: String): String? {
        val upgradeBehaviourValues = InstallerManifest.UpgradeBehavior.values().map { it.toString() }
        return when {
            option.firstOrNull()?.lowercaseChar() !in upgradeBehaviourValues.map { it.first().lowercaseChar() } -> {
                Errors.invalidEnum(upgradeBehaviourValues)
            }
            else -> null
        }
    }

    private const val upgradeBehaviourInfo = "${Prompts.optional} Enter the Upgrade Behavior"
}
