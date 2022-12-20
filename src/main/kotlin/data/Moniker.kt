package data

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.terminal.Terminal
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object Moniker : KoinComponent {
    fun Terminal.monikerPrompt() {
        val schemasImpl: SchemasImpl by inject()
        val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
        val monikerSchema = schemasImpl.defaultLocaleSchema.definitions.tag
        do {
            println(brightYellow(monikerInfo))
            val input = prompt(brightWhite(PromptType.Moniker.toString()))?.trim()
            val (packageLocaleValid, error) = isMonikerValid(input, monikerSchema)
            if (packageLocaleValid == Validation.Success && input != null) {
                defaultLocaleManifestData.moniker = input
            }
            error?.let { println(red(it)) }
            println()
        } while (packageLocaleValid != Validation.Success)
    }

    fun isMonikerValid(
        moniker: String?,
        monikerSchema: DefaultLocaleSchema.Definitions.Tag
    ): Pair<Validation, String?> {
        return when {
            !moniker.isNullOrBlank() &&
                (moniker.length < monikerSchema.minLength || moniker.length > monikerSchema.maxLength) -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = monikerSchema.minLength,
                    max = monikerSchema.maxLength
                )
            }
            else -> Validation.Success to null
        }
    }

    private const val monikerInfo = "${Prompts.optional} Enter the Moniker (friendly name/alias). For example: vscode"
}
