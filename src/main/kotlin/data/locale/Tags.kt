package data.locale

import Errors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import data.PreviousManifestData
import data.SharedManifestData
import input.ExitCode
import input.Prompts
import input.YamlExtensions.convertToYamlList
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import schemas.manifest.DefaultLocaleManifest
import kotlin.system.exitProcess

object Tags : KoinComponent, CommandPrompt<List<String>> {
    private val previousManifestData: PreviousManifestData by inject()
    private val sharedManifestData: SharedManifestData by inject()

    override suspend fun prompt(terminal: Terminal): List<String> = with(terminal) {
        return sharedManifestData.gitHubDetection?.topics ?: let {
            println(colors.brightYellow(tagsInfo))
            info(example)
            prompt(
                prompt = DefaultLocaleManifest::tags.name.replaceFirstChar { it.titlecase() },
                default = previousManifestData.remoteDefaultLocaleData.await()?.tags?.also {
                    muted("Previous tags: $it")
                },
                convert = { input ->
                    getError(input)
                        ?.let { ConversionResult.Invalid(it) }
                        ?: ConversionResult.Valid(input.trim().convertToYamlList(uniqueItems))
                }
            ) ?: exitProcess(ExitCode.CtrlC.code)
        }
    }

    override fun getError(input: String?): String? {
        val convertedInput = input?.trim()?.convertToYamlList(uniqueItems)
        return when {
            convertedInput == null -> null
            convertedInput.count() > maxCount -> Errors.invalidLength(max = maxCount)
            convertedInput.any { it.length > maxLength } -> {
                Errors.invalidLength(
                    min = minLength,
                    max = maxLength,
                    items = convertedInput.filter { it.length > maxLength }
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
    private const val uniqueItems = true
}
