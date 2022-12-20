package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import input.PromptType
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerSchema
import schemas.SchemasImpl

object InstallerSuccessCodes : KoinComponent {
    fun Terminal.installerSuccessCodesPrompt() {
        val installerManifestData: InstallerManifestData by inject()
        val schemasImpl: SchemasImpl by inject()
        val installerSuccessCodesSchema = schemasImpl.installerSchema.definitions.installerSuccessCodes
        val installerReturnCodeSchema = schemasImpl.installerSchema.definitions.installerReturnCode
        do {
            println(brightYellow(installerSuccessCodeInfo(installerSuccessCodesSchema)))
            val input = prompt(brightWhite(PromptType.InstallerSuccessCodes.toString()))
                ?.trim()
                ?.convertToYamlList(installerSuccessCodesSchema.uniqueItems)
                ?.mapNotNull { it.toIntOrNull() }
                ?.filterNot { it in installerReturnCodeSchema.not.enum }
            val (installerSuccessCodesValid, error) = areInstallerSuccessCodesValid(input)
            if (installerSuccessCodesValid == Validation.Success) {
                installerManifestData.installerSuccessCodes = input
            }
            error?.let { println(red(it)) }
            println()
        } while (installerSuccessCodesValid != Validation.Success)
    }

    fun areInstallerSuccessCodesValid(
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

    private fun installerSuccessCodeInfo(
        successCodesDefinitions: InstallerSchema.Definitions.InstallerSuccessCodes
    ): String {
        return "${Prompts.optional} ${successCodesDefinitions.description} (Max ${successCodesDefinitions.maxItems})"
    }
}
