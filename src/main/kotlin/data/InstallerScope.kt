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

object InstallerScope : KoinComponent {
    fun Terminal.installerScopePrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val installerSchemaImpl: InstallerSchemaImpl = get()
        var promptInput: String?
        val installerScopeEnum = Enum.installerScope(installerSchemaImpl.installerSchema)
        do {
            println(
                verticalLayout {
                    cell(brightYellow(Prompts.installerScopeInfo))
                    installerScopeEnum.forEach { scope ->
                        cell(
                            brightWhite(
                                buildString {
                                    append(" ".repeat(Prompts.optionIndent))
                                    append("[${scope.first().titlecase()}] ")
                                    append(scope.replaceFirstChar { it.titlecase() })
                                }
                            )
                        )
                    }
                    cell(
                        brightGreen(
                            buildString {
                                append(" ".repeat(Prompts.optionIndent))
                                append("[${Prompts.noIdea.first().titlecase()}] ")
                                append(Prompts.noIdea)
                            }
                        )
                    )
                }
            )
            promptInput = prompt(brightWhite(Prompts.enterChoice), default = Prompts.noIdea.first().titlecase())?.trim()
            val (installerScopeValid, error) = isInstallerScopeValid(promptInput?.firstOrNull())
            error?.let { println(red(it)) }
            println()
        } while (installerScopeValid != Validation.Success)
        installerManifestData.installerScope = installerScopeEnum.firstOrNull {
            it.firstOrNull()?.titlecase() == promptInput?.firstOrNull()?.titlecase()
        }
    }

    fun isInstallerScopeValid(
        option: Char?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerScopeEnum = Enum.installerScope(installerSchema)
        return when {
            option != Prompts.noIdea.first() && installerScopeEnum.all {
                it.first().titlecase() != option?.titlecase()
            } -> Validation.InvalidInstallerScope to Errors.invalidEnum(
                Validation.InvalidInstallerScope,
                installerScopeEnum
            )
            else -> Validation.Success to null
        }
    }
}
