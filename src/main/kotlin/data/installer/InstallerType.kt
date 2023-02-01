package data.installer

import Errors
import ExitCode
import Validation
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.PreviousManifestData
import data.SharedManifestData
import detection.files.msix.MsixBundle
import input.Prompts
import network.HttpUtils
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.manifest.InstallerManifest
import kotlin.system.exitProcess

object InstallerType : KoinComponent {
    private val installerManifestData: InstallerManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val installerTypeSchema = get<SchemasImpl>().installerSchema.definitions.installerType
    private val sharedManifestData: SharedManifestData by inject()

    fun Terminal.installerTypePrompt() {
        when (HttpUtils.getURLExtension(installerManifestData.installerUrl)) {
            InstallerManifest.InstallerType.MSIX.toString(), MsixBundle.msixBundleConst -> {
                installerManifestData.installerType = InstallerManifest.Installer.InstallerType.MSIX
            }
            InstallerManifest.InstallerType.MSI.toString() -> {
                installerManifestData.installerType = when (sharedManifestData.msi?.isWix) {
                    true -> InstallerManifest.Installer.InstallerType.WIX
                    else -> InstallerManifest.Installer.InstallerType.MSI
                }
            }
            InstallerManifest.Installer.InstallerType.ZIP.toString() -> {
                installerManifestData.installerType = InstallerManifest.Installer.InstallerType.ZIP
            }
            InstallerManifest.InstallerType.APPX.toString(), MsixBundle.appxBundleConst -> {
                installerManifestData.installerType = InstallerManifest.Installer.InstallerType.APPX
            }
            else -> {
                if (installerManifestData.installerType == null) {
                    installerTypeInfo().also { (info, infoColor) -> println(infoColor(info)) }
                    info("Options: ${installerTypeSchema.enum.joinToString(", ")}")
                    installerManifestData.installerType = prompt(
                        prompt = const,
                        default = getPreviousValue()?.toInstallerType()?.also { muted("Previous installer type: $it") },
                        convert = { input ->
                            isInstallerTypeValid(input)
                                ?.let { ConversionResult.Invalid(it) }
                                ?: ConversionResult.Valid(input.toInstallerType())
                        }
                    ) ?: exitProcess(ExitCode.CtrlC.code)
                    println()
                }
            }
        }
    }

    private fun isInstallerTypeValid(installerType: String): String? {
        return when {
            installerType.isBlank() -> Errors.blankInput(const)
            !installerTypeSchema.enum.contains(installerType) -> {
                Errors.invalidEnum(Validation.InvalidInstallerType, installerTypeSchema.enum)
            }
            else -> null
        }
    }

    private fun String.toInstallerType(): InstallerManifest.Installer.InstallerType {
        InstallerManifest.Installer.InstallerType.values().forEach { if (it.toString().lowercase() == this) return it }
        throw IllegalArgumentException("Invalid installer type: $this")
    }

    private fun getPreviousValue(): String? {
        return previousManifestData.remoteInstallerData?.let {
            it.installerType?.toString()
                ?: it.installers.getOrNull(installerManifestData.installers.size)?.installerType?.toString()
        }
    }

    private fun installerTypeInfo(): Pair<String, TextColors> {
        return buildString {
            append(if (getPreviousValue() == null) Prompts.required else Prompts.optional)
            append(" Enter the installer type")
        } to if (getPreviousValue() == null) brightGreen else brightYellow
    }

    const val const = "Installer Type"
}
