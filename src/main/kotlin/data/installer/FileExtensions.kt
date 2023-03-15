package data.installer

import commands.interfaces.ListPrompt
import commands.interfaces.ListValidationRules
import extensions.YamlExtensions.convertToList
import schemas.manifest.InstallerManifest

class FileExtensions(
    previousInstallerManifest: InstallerManifest?,
    private val installersSize: Int
) : ListPrompt<String> {
    override val name: String = "File extensions"

    override val description: String = "List of file extensions the package could support"

    override val extraText: String? = null

    override val validationRules: ListValidationRules<String> = ListValidationRules(
        maxItems = 512,
        maxItemLength = 64,
        minItemLength = 1,
        transform = ::convertToList,
        regex = Regex("^[^\\\\/:*?\"<>|\\x01-\\x1f]+$")
    )

    override val default: List<String>? = previousInstallerManifest?.run {
        fileExtensions ?: installers.getOrNull(installersSize)?.fileExtensions
    }
}
