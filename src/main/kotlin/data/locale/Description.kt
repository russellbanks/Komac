package data.locale

import Errors
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.ExitCode
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import kotlin.system.exitProcess

object Description : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()

    suspend fun Terminal.descriptionPrompt(descriptionType: DescriptionType) {
        if (
            descriptionType == DescriptionType.Short &&
            sharedManifestData.gitHubDetection?.shortDescription?.await() != null &&
            getPreviousValue(descriptionType) == null
        ) {
            defaultLocaleManifestData.shortDescription = sharedManifestData.gitHubDetection?.shortDescription?.await()!!
            return
        }
        do {
            val textColour = if (descriptionType == DescriptionType.Short) brightGreen else brightYellow
            println(textColour(descriptionInfo(descriptionType)))
            sharedManifestData.msix?.description?.let { println(cyan("Description from installer: $it")) }
            val input = prompt(
                prompt = descriptionType.promptName,
                default = getPreviousValue(descriptionType)?.also {
                    muted("Previous ${descriptionType.name.lowercase()}: $it")
                }
            )?.trim() ?: exitProcess(ExitCode.CtrlC.code)
            val error = descriptionValid(
                description = input,
                descriptionType = descriptionType,
                canBeBlank = descriptionType == DescriptionType.Long
            )
            when (descriptionType) {
                DescriptionType.Short -> defaultLocaleManifestData.shortDescription = input
                DescriptionType.Long -> defaultLocaleManifestData.description = input
            }
            error?.let { println(brightRed(it)) }
            println()
        } while (error != null)
    }

    private fun descriptionValid(
        description: String?,
        descriptionType: DescriptionType,
        canBeBlank: Boolean
    ): String? {
        return when {
            description.isNullOrBlank() && canBeBlank -> null
            description.isNullOrBlank() -> Errors.blankInput(descriptionType)
            description.length < descriptionType.minLength || description.length > descriptionType.maxLength -> {
                Errors.invalidLength(min = descriptionType.minLength, max = descriptionType.maxLength)
            }
            else -> null
        }
    }

    private suspend fun getPreviousValue(descriptionType: DescriptionType): String? {
        val remoteDefaultLocaleData = previousManifestData.remoteDefaultLocaleData
        return when (descriptionType) {
            DescriptionType.Short -> remoteDefaultLocaleData.await()?.shortDescription
            DescriptionType.Long -> remoteDefaultLocaleData.await()?.description
        }
    }

    private fun descriptionInfo(descriptionType: DescriptionType): String {
        val description = when (descriptionType) {
            DescriptionType.Short -> "The short package description"
            DescriptionType.Long -> "The full package description"
        }
        val inputNecessary = if (descriptionType == DescriptionType.Short) Prompts.required else Prompts.optional
        return "$inputNecessary Enter ${description.lowercase()}"
    }
}
