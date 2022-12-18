package data

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
import schemas.Enum
import schemas.InstallerSchema
import schemas.InstallerSchemaImpl

object UpgradeBehaviour : KoinComponent {
    fun Terminal.upgradeBehaviourPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val installerSchemaImpl: InstallerSchemaImpl = get()
        var promptInput: String?
        val upgradeBehaviourEnum = Enum.upgradeBehaviour(installerSchemaImpl.installerSchema)
        do {
            println(
                verticalLayout {
                    cell(brightYellow(Prompts.upgradeBehaviourInfo))
                    upgradeBehaviourEnum.forEach { behaviour ->
                        val textColour = when {
                            behaviour.first().titlecase() == upgradeBehaviourEnum.first().first().titlecase() -> {
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
                default = upgradeBehaviourEnum.first().first().titlecase()
            )?.trim()
            val (upgradeBehaviourValid, error) = isUpgradeBehaviourValid(
                promptInput?.firstOrNull()
            )
            error?.let { println(red(it)) }
            println()
        } while (upgradeBehaviourValid != Validation.Success)
        installerManifestData.upgradeBehavior = upgradeBehaviourEnum.firstOrNull {
            it.firstOrNull()?.titlecase() == promptInput?.firstOrNull()?.titlecase()
        }
    }

    fun isUpgradeBehaviourValid(
        option: Char?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val upgradeBehaviourEnum = Enum.upgradeBehaviour(installerSchema)
        return when {
            upgradeBehaviourEnum.all {
                it.first().titlecase() != option?.titlecase()
            } -> Validation.InvalidUpgradeBehaviour to Errors.invalidEnum(
                Validation.InvalidUpgradeBehaviour,
                upgradeBehaviourEnum
            )
            else -> Validation.Success to null
        }
    }
}
