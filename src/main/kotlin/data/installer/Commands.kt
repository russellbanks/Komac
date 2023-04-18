package data.installer

import commands.interfaces.ListPrompt
import commands.interfaces.ListValidationRules
import data.AllManifestData
import data.PreviousManifestData
import extensions.YamlExtensions.convertToList

object Commands : ListPrompt<String> {
    override val name: String = "Commands"

    override val description: String = "List of commands or aliases to run the package"

    override val extraText: String? = null

    override val validationRules: ListValidationRules<String> = ListValidationRules(
        maxItems = 16,
        minItemLength = 1,
        maxItemLength = 40,
        transform = ::convertToList
    )

    override val default: List<String>? get() = PreviousManifestData.installerManifest?.run {
        commands ?: installers.getOrNull(AllManifestData.installers.size)?.commands
    }
}
