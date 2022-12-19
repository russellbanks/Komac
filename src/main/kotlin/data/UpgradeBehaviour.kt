package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object UpgradeBehaviour : KoinComponent {
    fun Terminal.upgradeBehaviourPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val schemasImpl: SchemasImpl = get()
        val upgradeBehaviourSchema = schemasImpl.installerSchema.definitions.upgradeBehavior
        var promptInput: String?
        do {
            println(
                verticalLayout {
                    cell(brightYellow(upgradeBehaviourInfo))
                    upgradeBehaviourSchema.enum.forEach { behaviour ->
                        val textColour = when {
                            behaviour.first().titlecase() ==
                                upgradeBehaviourSchema.enum.first().first().titlecase() -> {
                                brightGreen
                            }
                            else -> brightWhite
                        }
                        cell(
                            textColour(
                                buildString {
                                    append(" ".repeat(Prompts.optionIndent))
                                    append("[${behaviour.first().titlecase()}] ")
                                    append(behaviour.replaceFirstChar { it.titlecase() })
                                }
                            )
                        )
                    }
                }
            )
            promptInput = prompt(
                prompt = brightWhite(Prompts.enterChoice),
                default = upgradeBehaviourSchema.enum.first().first().titlecase()
            )?.trim()
            val (upgradeBehaviourValid, error) = isUpgradeBehaviourValid(
                promptInput?.firstOrNull(),
                upgradeBehaviourSchema
            )
            error?.let { println(red(it)) }
            println()
        } while (upgradeBehaviourValid != Validation.Success)
        installerManifestData.upgradeBehavior = upgradeBehaviourSchema.enum.firstOrNull {
            it.firstOrNull()?.titlecase() == promptInput?.firstOrNull()?.titlecase()
        }?.toUpgradeBehaviour()
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

    private fun String.toUpgradeBehaviour(): InstallerManifest.UpgradeBehavior? {
        return InstallerManifest.UpgradeBehavior.values().firstOrNull { it.name.lowercase() == lowercase() }
    }

    private const val upgradeBehaviourInfo = "${Prompts.optional} Enter the Upgrade Behavior"
}
