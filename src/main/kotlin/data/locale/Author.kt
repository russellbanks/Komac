package data.locale

import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object Author : KoinComponent {
    fun Terminal.authorPrompt() {
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        val authorSchema = get<SchemasImpl>().defaultLocaleSchema.properties.author
        do {
            println(brightYellow(authorInfo(authorSchema)))
            val input = prompt(brightWhite(PromptType.Author.toString()))?.trim()
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

    private fun authorInfo(authorSchema: DefaultLocaleSchema.Properties.Author): String {
        return "${Prompts.optional} Enter ${authorSchema.description.lowercase()}"
    }
}
