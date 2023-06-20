package data.installer

import io.menu.prompts.ListPrompt
import io.menu.prompts.ListValidationRules
import schemas.manifest.InstallerManifest
import schemas.manifest.YamlExtensions.convertToList

class Commands(
    private val currentInstallerIndex: Int,
    private val previousInstallerManifest: InstallerManifest?
) : ListPrompt<String> {
    override val name: String = "Commands"

    override val description: String = "List of commands or aliases to run the package"

    override val extraText: String? = null

    override val validationRules: ListValidationRules<String> = ListValidationRules(
        maxItems = 16,
        minItemLength = 1,
        maxItemLength = 40,
        transform = ::convertToList
    )

    override val default: List<String>? get() = previousInstallerManifest?.run {
        commands ?: installers.getOrNull(currentInstallerIndex)?.commands
    }
}
