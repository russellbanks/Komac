package data.locale

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.SharedManifestData
import input.PromptType
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object Tags : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private val schemasImpl: SchemasImpl by inject()
    private val tagsSchema = schemasImpl.defaultLocaleSchema.properties.tags
    private val tagSchema = schemasImpl.defaultLocaleSchema.definitions.tag

    suspend fun Terminal.tagsPrompt() {
        do {
            println(brightYellow(tagsInfo))
            println(cyan(tagsExample))
            val input = prompt(
                prompt = brightWhite(PromptType.Tags.toString()),
                default = sharedManifestData.remoteDefaultLocaleData.await()?.tags?.joinToString(", ").also {
                    println(gray("Previous tags: $it"))
                }
            )?.trim()?.convertToYamlList(tagsSchema.uniqueItems)
            val (commandsValid, error) = areTagsValid(input, tagsSchema, tagSchema)
            if (commandsValid == Validation.Success) {
                defaultLocaleManifestData.tags = input
            }
            error?.let { println(red(it)) }
            println()
        } while (commandsValid != Validation.Success)
    }

    fun areTagsValid(
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

    private const val tagsExample = "Example: zip, c++, photos, OBS"
}
