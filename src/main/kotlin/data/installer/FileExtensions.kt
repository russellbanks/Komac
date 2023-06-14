package data.installer

import io.menu.prompts.ListPrompt
import io.menu.prompts.ListValidationRules
import schemas.manifest.InstallerManifest
import schemas.manifest.YamlExtensions.convertToList

class FileExtensions(
    private val currentInstallerIndex: Int,
    private val previousInstallerManifest: InstallerManifest?
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

    override val default: List<String>? get() = previousInstallerManifest?.run {
        fileExtensions ?: installers.getOrNull(currentInstallerIndex)?.fileExtensions
    }
}
