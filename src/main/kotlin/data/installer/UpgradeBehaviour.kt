package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.manifest.InstallerManifest

object UpgradeBehaviour : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val upgradeBehaviourSchema = schemasImpl.installerSchema.definitions.upgradeBehavior

    fun Terminal.upgradeBehaviourPrompt() {
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
                isUpgradeBehaviourValid(input.firstOrNull())
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )
        installerManifestData.upgradeBehavior = InstallerManifest.UpgradeBehavior.values().firstOrNull {
            it.name.firstOrNull()?.titlecase() == input?.firstOrNull()?.titlecase()
        }
        println()
    }

    private fun getPreviousValue(): Enum<*>? {
        return previousManifestData.remoteInstallerData?.let {
            it.upgradeBehavior ?: it.installers.getOrNull(installerManifestData.installers.size)?.upgradeBehavior
        }
    }

    private fun isUpgradeBehaviourValid(option: Char?): String? {
        return when {
            upgradeBehaviourSchema.enum.all {
                it.first().titlecase() != option?.titlecase()
            } -> Errors.invalidEnum(
                Validation.InvalidUpgradeBehaviour,
                upgradeBehaviourSchema.enum
            )
            else -> null
        }
    }

    private const val upgradeBehaviourInfo = "${Prompts.optional} Enter the Upgrade Behavior"
}
