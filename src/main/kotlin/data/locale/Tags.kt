package data.locale

import Errors
import Validation
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.DefaultLocaleSchema

object Tags : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val tagsSchema = schemasImpl.defaultLocaleSchema.properties.tags
    private val tagSchema = schemasImpl.defaultLocaleSchema.definitions.tag

    suspend fun Terminal.tagsPrompt() {
        sharedManifestData.gitHubDetection?.topics?.await()?.let {
            defaultLocaleManifestData.tags = it
            return
        }
        do {
            println(colors.brightYellow(tagsInfo))
            info(example)
            val input = prompt(
                prompt = const,
                default = previousManifestData.remoteDefaultLocaleData?.tags?.joinToString(", ")?.also {
                    muted("Previous tags: $it")
                }
            )?.trim()?.convertToYamlList(tagsSchema.uniqueItems)
            val (commandsValid, error) = areTagsValid(input, tagsSchema, tagSchema)
            if (commandsValid == Validation.Success) {
                defaultLocaleManifestData.tags = input
            }
            error?.let { danger(it) }
            println()
        } while (commandsValid != Validation.Success)
    }

    private fun areTagsValid(
        tags: Iterable<String>?,
        tagsSchema: DefaultLocaleSchema.Properties.Tags,
        tagSchema: DefaultLocaleSchema.Definitions.Tag
    ): Pair<Validation, String?> {
        return when {
            (tags?.count() ?: 0) > tagsSchema.maxItems -> {
                Validation.InvalidLength to Errors.invalidLength(max = tagsSchema.maxItems)
            }
            tags?.any { it.length > tagSchema.maxLength } == true -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = tagSchema.minLength,
                    max = tagSchema.maxLength,
                    items = tags.filter { it.length > tagSchema.maxLength }
                )
            }
            else -> Validation.Success to null
        }
    }

    private val tagsInfo = buildString {
        append(Prompts.optional)
        append(" Enter any tags that would be useful to discover this tool. ")
        append("(Max ${tagsSchema.maxItems})")
    }

    private const val const = "Tags"
    private const val example = "Example: zip, c++, photos, OBS"
}
