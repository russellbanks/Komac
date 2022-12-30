package data.installer

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.InstallerManifest
import schemas.InstallerSchema
import schemas.SchemasImpl

object InstallerType : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val installerTypeSchema = get<SchemasImpl>().installerSchema.definitions.installerType

    fun Terminal.installerTypePrompt() {
        do {
            installerTypeInfo().also { (info, infoColor) -> println(infoColor(info)) }
            println(cyan("Options: ${installerTypeSchema.enum.joinToString(", ")}"))
            val input = prompt(
                prompt = brightWhite(PromptType.InstallerType.toString()),
                default = getPreviousValue()?.also { println(gray("Previous installer type: $it")) }
            )?.trim()?.lowercase()
            val (installerTypeValid, error) = isInstallerTypeValid(input, installerTypeSchema)
            error?.let { println(red(it)) }
            if (installerTypeValid == Validation.Success && input != null) {
                installerManifestData.installerType = input.toInstallerType()
            }
            println()
        } while (installerTypeValid != Validation.Success)
    }

    fun isInstallerTypeValid(
        installerType: String?,
        schema: InstallerSchema.Definitions.InstallerType = installerTypeSchema
    ): Pair<Validation, String?> {
        return when {
            installerType.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.InstallerType)
            !schema.enum.contains(installerType) -> {
                Validation.InvalidInstallerType to Errors.invalidEnum(Validation.InvalidInstallerType, schema.enum)
            }
            else -> Validation.Success to null
        }
    }

    private fun String.toInstallerType(): InstallerManifest.InstallerType {
        InstallerManifest.InstallerType.values().forEach {
            if (it.toString().lowercase() == this) return it
        }
        throw IllegalArgumentException("Invalid installer type: $this")
    }

    private fun getPreviousValue(): String? {
        return previousManifestData.remoteInstallerData?.let {
            it.installerType?.toString()
                ?: it.installers[installerManifestData.installers.size].installerType?.toString()
        }
    }

    private fun installerTypeInfo(): Pair<String, TextColors> {
        return buildString {
            append(if (getPreviousValue() == null) Prompts.required else Prompts.optional)
            append(" Enter the installer type")
        } to if (getPreviousValue() == null) brightGreen else brightYellow
    }
}
