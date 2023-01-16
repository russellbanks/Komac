package data.locale

import Errors
import Validation
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightRed
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.brightYellow
import com.github.ajalt.mordant.rendering.TextColors.cyan
import com.github.ajalt.mordant.terminal.Terminal
import data.DefaultLocaleManifestData
import data.PreviousManifestData
import data.SharedManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import schemas.data.DefaultLocaleSchema

object Description : KoinComponent {
    private val defaultLocaleManifestData: DefaultLocaleManifestData by inject()
    private val previousManifestData: PreviousManifestData by inject()
    private val propertiesSchema: DefaultLocaleSchema.Properties = get<SchemasImpl>().defaultLocaleSchema.properties
    private val sharedManifestData: SharedManifestData by inject()

    fun Terminal.descriptionPrompt(descriptionType: DescriptionType) {
        do {
            val textColour = if (descriptionType == DescriptionType.Short) brightGreen else brightYellow
            println(textColour(descriptionInfo(descriptionType)))
            sharedManifestData.msix?.description?.let { println(cyan("Description from installer: $it")) }
            val input = prompt(
                prompt = brightWhite(descriptionType.promptName),
                default = getPreviousValue(descriptionType)?.also {
                    println(brightWhite("Previous ${descriptionType.name.lowercase()}: $it"))
                }
            )?.trim()
            val (descriptionValid, error) = descriptionValid(
                description = input,
                descriptionType = descriptionType,
                propertiesSchema = propertiesSchema,
                canBeBlank = descriptionType == DescriptionType.Long
            )
            if (descriptionValid == Validation.Success && input != null) {
                when (descriptionType) {
                    DescriptionType.Short -> defaultLocaleManifestData.shortDescription = input
                    DescriptionType.Long -> defaultLocaleManifestData.description = input
                }
            }
            error?.let { println(brightRed(it)) }
            println()
        } while (descriptionValid != Validation.Success)
    }

    fun descriptionValid(
        description: String?,
        descriptionType: DescriptionType,
        propertiesSchema: DefaultLocaleSchema.Properties,
        canBeBlank: Boolean
    ): Pair<Validation, String?> {
        val minLength = when (descriptionType) {
            DescriptionType.Short -> propertiesSchema.shortDescription.minLength
            DescriptionType.Long -> propertiesSchema.description.minLength
        }
        val maxLength = when (descriptionType) {
            DescriptionType.Short -> propertiesSchema.shortDescription.maxLength
            DescriptionType.Long -> propertiesSchema.description.maxLength
        }
        return when {
            description.isNullOrBlank() && canBeBlank -> Validation.Success to null
            description.isNullOrBlank() -> Validation.Blank to Errors.blankInput(descriptionType)
            description.length < minLength || description.length > maxLength -> {
                Validation.InvalidLength to Errors.invalidLength(min = minLength, max = maxLength)
            }
            else -> Validation.Success to null
        }
    }

    private fun getPreviousValue(descriptionType: DescriptionType): String? {
        val remoteDefaultLocaleData = previousManifestData.remoteDefaultLocaleData
        return when (descriptionType) {
            DescriptionType.Short -> remoteDefaultLocaleData?.shortDescription
            DescriptionType.Long -> remoteDefaultLocaleData?.description
        }
    }

    private fun descriptionInfo(descriptionType: DescriptionType): String {
        val description = when (descriptionType) {
            DescriptionType.Short -> propertiesSchema.shortDescription.description
            DescriptionType.Long -> propertiesSchema.description.description
        }
        val inputNecessary = if (descriptionType == DescriptionType.Short) Prompts.required else Prompts.optional
        return "$inputNecessary Enter ${description.lowercase()}"
    }
}
