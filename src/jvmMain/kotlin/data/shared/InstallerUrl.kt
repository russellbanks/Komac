package data.shared

import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.rendering.TextStyles
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import io.menu.prompts.UrlPrompt
import commands.prompts.validation.UrlValidationRules
import utils.msix.MsixBundle

object InstallerUrl : UrlPrompt {
    override val name: String = "Installer URL"

    override val description: String = "Download URL to the installer"

    override val validationRules: UrlValidationRules = UrlValidationRules(
        isRequired = true
    )

    fun Terminal.msixBundleDetection(msixBundle: MsixBundle?) {
        if (msixBundle != null) {
            println(
                verticalLayout {
                    cell(
                        (TextColors.brightGreen + TextStyles.bold)(
                            "${msixBundle.packages?.size} packages have been detected inside the MSIX Bundle:"
                        )
                    )
                    msixBundle.packages?.forEachIndexed { index, individualPackage ->
                        cell(TextColors.brightGreen("Package ${index.inc()}/${msixBundle.packages?.size}"))
                        listOf(
                            "Architecture" to individualPackage.processorArchitecture,
                            "Version" to individualPackage.version,
                            "Minimum version" to individualPackage.minVersion,
                            "utils.Platform" to individualPackage.targetDeviceFamily
                        ).forEach { (text, value) ->
                            if (value != null) {
                                var newText = text
                                var newValue = value
                                if (value is List<*>) {
                                    if (value.size > 1) newText = "${text}s"
                                    newValue = value.joinToString()
                                }
                                cell(TextColors.brightWhite("${" ".repeat(3)} $newText: $newValue"))
                            }
                        }
                    }
                }
            )
            println()
            info("All packages inside the MSIX Bundle will be added as separate installers in the manifest")
            println()
        }
    }
}
