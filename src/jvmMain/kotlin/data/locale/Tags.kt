package data.locale

import commands.prompts.ListPrompt
import commands.prompts.validation.ListValidationRules
import schemas.manifest.DefaultLocaleManifest
import extensions.YamlExtensions.convertToList

class Tags(private val previousDefaultLocaleManifest: DefaultLocaleManifest?) : ListPrompt<String> {
    override val name = "Tags"

    override val extraText: String = "Example: zip, c++, photos, OBS"

    override val description: String = "tags that would be useful to discover this tool"

    override val default: List<String>? get() = previousDefaultLocaleManifest?.tags

    override val validationRules: ListValidationRules<String> = Tags.validationRules

    companion object {
        val validationRules: ListValidationRules<String> = ListValidationRules(
            maxItems = 16,
            minItemLength = 1,
            maxItemLength = 40,
            transform = ::convertToList
        )
    }
}
