package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.PromptType
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.InstallerSchema

object InstallerSuccessCodes : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val installerSuccessCodesSchema = schemasImpl.installerSchema.definitions.installerSuccessCodes
    private val installerReturnCodeSchema = schemasImpl.installerSchema.definitions.installerReturnCode

    fun Terminal.installerSuccessCodesPrompt() {
        do {
            println(brightYellow(installerSuccessCodeInfo))
            println(cyan(installerSuccessCodesExample))
            val input = prompt(
                prompt = brightWhite(PromptType.InstallerSuccessCodes.toString()),
                default = getPreviousValue()?.joinToString(", ")?.also {
                    println(gray("Previous commands: $it"))
                }
            )?.trim()
                ?.convertToYamlList(installerSuccessCodesSchema.uniqueItems)
                ?.mapNotNull { it.toIntOrNull() }
                ?.filterNot { it in installerReturnCodeSchema.not.enum }
            val (installerSuccessCodesValid, error) = areInstallerSuccessCodesValid(input)
            if (installerSuccessCodesValid == Validation.Success) {
                installerManifestData.installerSuccessCodes = input
            }
            error?.let { println(brightRed(it)) }
            println()
        } while (installerSuccessCodesValid != Validation.Success)
    }

    private fun areInstallerSuccessCodesValid(
        installerSuccessCodes: Iterable<Int>?,
        installerSchema: InstallerSchema = get<SchemasImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerSuccessCodesSchema = installerSchema.definitions.installerSuccessCodes
        val installerReturnCodeSchema = installerSchema.definitions.installerReturnCode
        return when {
            (installerSuccessCodes?.count() ?: 0) > installerSuccessCodesSchema.maxItems -> {
                Validation.InvalidLength to Errors.invalidLength(max = installerSuccessCodesSchema.maxItems)
            }
            installerSuccessCodes?.any {
                it < installerReturnCodeSchema.minimum || it > installerReturnCodeSchema.maximum
            } == true -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = installerReturnCodeSchema.minimum,
                    max = installerReturnCodeSchema.maximum,
                    items = installerSuccessCodes.filter {
                        it < installerReturnCodeSchema.minimum || it > installerReturnCodeSchema.maximum
                    }.map { it.toString() }
                )
            }
            else -> Validation.Success to null
        }
    }

    private fun generateRandomInstallerSuccessCodes(): List<Int> {
        val installerSuccessCodes = listOf(13, 87, 120, 1259, 3010) + IntRange(1601, 1616) + IntRange(1618, 1654)
        return installerSuccessCodes.shuffled().take(3).sorted()
    }

    private fun getPreviousValue(): List<Int>? {
        return previousManifestData.remoteInstallerData?.let {
            it.installerSuccessCodes ?: it.installers[installerManifestData.installers.size].installerSuccessCodes
        }
    }

    private val installerSuccessCodeInfo =
        "${Prompts.optional} ${installerSuccessCodesSchema.description} (Max ${installerSuccessCodesSchema.maxItems})"
    private val installerSuccessCodesExample =
        "Example: ${generateRandomInstallerSuccessCodes().joinToString(", ")}"
}
