package data.locale

import Errors
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.DefaultLocaleSchema

object Moniker : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()

    fun Terminal.monikerPrompt() {
        println(colors.brightYellow(monikerInfo))
        info(monikerExample)
        defaultLocaleManifestData.moniker = prompt(
            prompt = colors.brightWhite(const),
            default = previousManifestData.remoteDefaultLocaleData?.moniker
                ?.also { muted("Previous moniker: $it") },
            convert = {
                val error = isMonikerValid(it)
                if (error != null) {
                    ConversionResult.Invalid(error.message!!)
                } else {
                    ConversionResult.Valid(it)
                }
            }
        )!!.trim()
        println()
    }

    private fun isMonikerValid(
        moniker: String,
        monikerSchema: DefaultLocaleSchema.Definitions.Tag = get<SchemasImpl>().defaultLocaleSchema.definitions.tag
    ): CliktError? {
        return when {
            moniker.isNotBlank() &&
                (moniker.length < monikerSchema.minLength || moniker.length > monikerSchema.maxLength) -> {
                CliktError(Errors.invalidLength(min = monikerSchema.minLength, max = monikerSchema.maxLength))
            }
            else -> null
        }
    }

    private const val const = "Moniker"
    private const val monikerInfo = "${Prompts.optional} Enter the Moniker (friendly name/alias)."
    private const val monikerExample = "Example: vscode"
}
