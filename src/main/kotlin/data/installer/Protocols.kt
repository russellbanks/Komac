package data.installer

import commands.interfaces.ListPrompt
import commands.interfaces.ListValidationRules
import extensions.YamlExtensions.convertToList
import schemas.manifest.InstallerManifest

class Protocols(previousInstallerManifest: InstallerManifest?, private val installersSize: Int) : ListPrompt<String> {
    override val name: String = "Protocols"

    override val validationRules: ListValidationRules<String> = ListValidationRules(
        maxItems = 64,
        maxItemLength = 2048,
        transform = ::convertToList
    )

    override val description = "List of protocols the package provides a handler for"

    override val extraText: String? = null

    override val default: List<String>? = previousInstallerManifest?.run {
        protocols ?: installers.getOrNull(installersSize)?.protocols
    }
}
