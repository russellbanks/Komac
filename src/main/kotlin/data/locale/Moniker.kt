package data.locale

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.rendering.TextColors.gray
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import input.PromptType
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.DefaultLocaleSchema
import schemas.SchemasImpl

object Moniker : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val monikerSchema = get<SchemasImpl>().defaultLocaleSchema.definitions.tag

    fun Terminal.monikerPrompt() {
        do {
            println(brightYellow(monikerInfo))
            println(cyan(monikerExample))
            val input = prompt(
                prompt = brightWhite(PromptType.Moniker.toString()),
                default = previousManifestData.remoteDefaultLocaleData?.moniker?.also {
                    println(gray("Previous moniker: $it"))
                }
            )?.trim()
            val (packageLocaleValid, error) = isMonikerValid(input, monikerSchema)
            if (packageLocaleValid == Validation.Success && input != null) {
                defaultLocaleManifestData.moniker = input
            }
            error?.let { println(brightRed(it)) }
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

    private const val monikerInfo = "${Prompts.optional} Enter the Moniker (friendly name/alias)."
    private const val monikerExample = "Example: vscode"
}
