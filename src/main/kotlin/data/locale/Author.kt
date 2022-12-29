package data.locale

import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.SharedManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object Author : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()
    private val authorSchema = get<SchemasImpl>().defaultLocaleSchema.properties.author

    fun Terminal.authorPrompt() {
        do {
            println(brightYellow(authorInfo))
            val input = prompt(
                prompt = brightWhite(PromptType.Author.toString()),
                default = sharedManifestData.remoteDefaultLocaleData?.author?.also {
                    println(gray("Previous author: $it"))
                }
            )?.trim()
            val (packageLocaleValid, error) = isAuthorValid(input, authorSchema)
            if (packageLocaleValid == Validation.Success && input != null) {
                defaultLocaleManifestData.author = input
            }
            error?.let { println(red(it)) }
            println()
        } while (packageLocaleValid != Validation.Success)
    }

    fun isAuthorValid(
        author: String?,
        authorSchema: DefaultLocaleSchema.Properties.Author
    ): Pair<Validation, String?> {
        return when {
            !author.isNullOrBlank() &&
                (author.length < authorSchema.minLength || author.length > authorSchema.maxLength) -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = authorSchema.minLength,
                    max = authorSchema.maxLength
                )
            }
            else -> Validation.Success to null
        }
    }

    private val authorInfo = "${Prompts.optional} Enter ${authorSchema.description.lowercase()}"
}
