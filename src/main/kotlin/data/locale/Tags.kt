package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.DefaultLocaleManifest

object Tags : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()

    suspend fun Terminal.tagsPrompt() {
        sharedManifestData.gitHubDetection?.topics?.await()?.let {
            defaultLocaleManifestData.tags = it
            return
        }
        println(colors.brightYellow(tagsInfo))
        info(example)
        defaultLocaleManifestData.tags = prompt(
            prompt = DefaultLocaleManifest::tags.name.replaceFirstChar { it.titlecase() },
            default = previousManifestData.remoteDefaultLocaleData?.tags?.joinToString(", ")?.also {
                muted("Previous tags: $it")
            },
            convert = { input ->
                areTagsValid(input.trim().convertToYamlList(true))
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        )?.convertToYamlList(true)
        println()
    }

    private fun areTagsValid(tags: Iterable<String>): String? {
        return when {
            tags.count() > maxCount -> Errors.invalidLength(max = maxCount)
            tags.any { it.length > maxLength } -> {
                Errors.invalidLength(
                    min = minLength,
                    max = maxLength,
                    items = tags.filter { it.length > maxLength }
                )
            }
            else -> null
        }
    }

    private val tagsInfo = buildString {
        append(Prompts.optional)
        append(" Enter any tags that would be useful to discover this tool. ")
        append("(Max $maxCount)")
    }

    private const val example = "Example: zip, c++, photos, OBS"
    private const val maxCount = 16
    private const val maxLength = 40
    private const val minLength = 1
}
