package data.locale

import commands.interfaces.ListPrompt
import commands.interfaces.ListValidationRules
import extensions.YamlExtensions.convertToList

class Tags(previousTags: List<String>?) : ListPrompt<String> {
    override val name = "Tags"

    override val extraText: String = "Example: zip, c++, photos, OBS"

    override val description: String = "tags that would be useful to discover this tool"

    override val default: List<String>? = previousTags

    override val validationRules: ListValidationRules<String> = ListValidationRules(
        maxItems = 16,
        minItemLength = 1,
        maxItemLength = 40,
        transform = ::convertToList
    )
}
