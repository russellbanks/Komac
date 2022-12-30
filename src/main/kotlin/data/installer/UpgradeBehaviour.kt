package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object UpgradeBehaviour : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val upgradeBehaviourSchema = schemasImpl.installerSchema.definitions.upgradeBehavior

    fun Terminal.upgradeBehaviourPrompt() {
        do {
            val previousValue = getPreviousValue()
            println(
                verticalLayout {
                    cell(brightYellow(upgradeBehaviourInfo))
                    InstallerManifest.UpgradeBehavior.values().forEach { behaviour ->
                        val textColour = when {
                            previousValue == behaviour || previousValue == behaviour.toPerInstallerType() -> brightGreen
                            behaviour == InstallerManifest.UpgradeBehavior.Install -> brightGreen
                            else -> brightWhite
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
                    previousValue?.let { cell(gray("Previous upgrade behaviour: $previousValue")) }
                }
            )
            val input = prompt(
                prompt = brightWhite(Prompts.enterChoice),
                default = previousValue?.toString()?.firstOrNull()?.toString()
                    ?: InstallerManifest.UpgradeBehavior.Install.toString().first().toString()
            )?.trim()
            val (upgradeBehaviourValid, error) = isUpgradeBehaviourValid(input?.firstOrNull(), upgradeBehaviourSchema)
            if (upgradeBehaviourValid == Validation.Success) {
                installerManifestData.upgradeBehavior = InstallerManifest.UpgradeBehavior.values().firstOrNull {
                    it.name.firstOrNull()?.titlecase() == input?.firstOrNull()?.titlecase()
                }
            }
            error?.let { println(red(it)) }
            println()
        } while (upgradeBehaviourValid != Validation.Success)
    }

    private fun getPreviousValue(): Enum<*>? {
        return previousManifestData.remoteInstallerData?.let {
            it.upgradeBehavior ?: it.installers[installerManifestData.installers.size].upgradeBehavior
        }
    }

    fun isUpgradeBehaviourValid(
        option: Char?,
        upgradeBehaviourSchema: InstallerSchema.Definitions.UpgradeBehavior
    ): Pair<Validation, String?> {
        return when {
            upgradeBehaviourSchema.enum.all {
                it.first().titlecase() != option?.titlecase()
            } -> Validation.InvalidUpgradeBehaviour to Errors.invalidEnum(
                Validation.InvalidUpgradeBehaviour,
                upgradeBehaviourSchema.enum
            )
            else -> Validation.Success to null
        }
    }

    private const val upgradeBehaviourInfo = "${Prompts.optional} Enter the Upgrade Behavior"
}
