package data.installer

import io.menu.prompts.ListPrompt
import io.menu.prompts.ListValidationRules
import schemas.manifest.InstallerManifest
import schemas.manifest.YamlExtensions.convertToList

class Protocols(
    private val installersSize: Int,
    private val previousInstallerManifest: InstallerManifest?
) : ListPrompt<String> {
    override val name: String = "Protocols"

    override val validationRules: ListValidationRules<String> = ListValidationRules(
        maxItems = 64,
        maxItemLength = 2048,
        transform = ::convertToList
    )

    override val description: String = "List of protocols the package provides a handler for"

    override val extraText: String? = null

    override val default: List<String>? get() = previousInstallerManifest?.run {
        protocols ?: installers.getOrNull(installersSize)?.protocols
    }
}
