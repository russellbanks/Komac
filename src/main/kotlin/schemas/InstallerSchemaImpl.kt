package schemas

import Errors
import InstallerSwitch
import Ktor.isRedirect
import Validation
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.red
import input.PromptType
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.statement.HttpResponse
import io.ktor.http.HttpStatusCode
import io.ktor.http.isSuccess
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException
import java.util.Locale

@Single
class InstallerSchemaImpl : KoinComponent {
    private val retrieveSchemas: RetrieveSchemas = get()
    private val client: HttpClient = retrieveSchemas.client
    var installerSchema: InstallerSchema? = null
    private val terminalInstance: TerminalInstance by inject()
    private val progress = terminalInstance.terminal.progressAnimation {
        text("Retrieving installer schema")
        progressBar()
    }

    private var asyncJob: Deferred<Unit> = CoroutineScope(Dispatchers.Default).async {
        client.get(Schemas.installerSchema).body<String?>()?.let {
            val json = Json { ignoreUnknownKeys = true }
            installerSchema = json.decodeFromString(it)
        }
    }

    private suspend fun awaitInstallerSchema() {
        with(asyncJob) {
            if (isActive) {
                progress.run {
                    start()
                    invokeOnCompletion {
                        stop()
                        clear()
                    }
                    await()
                }
            }
        }
    }

    suspend fun isPackageIdentifierValid(
        identifier: String?,
        installerSchemaObj: InstallerSchema? = installerSchema
    ): Validation {
        awaitInstallerSchema()
        val packageIdentifierMaxLength = installerSchemaObj?.definitions?.packageIdentifier?.maxLength as Int
        with(terminalInstance.terminal) {
            return when {
                identifier.isNullOrBlank() -> Validation.Blank.also {
                    println(red(Errors.blankInput(PromptType.PackageVersion)))
                }
                identifier.length > packageIdentifierMaxLength -> Validation.InvalidLength.also {
                    println(
                        red(Errors.invalidLength(min = packageIdentifierMinLength, max = packageIdentifierMaxLength))
                    )
                }
                !identifier.matches(Pattern.packageIdentifier) -> Validation.InvalidPattern.also {
                    println(red(Errors.invalidRegex(Pattern.packageIdentifier)))
                }
                else -> Validation.Success
            }
        }
    }

    fun isPackageVersionValid(version: String?, installerSchemaObj: InstallerSchema? = installerSchema): Validation {
        val packageVersionMaxLength = installerSchemaObj?.definitions?.packageVersion?.maxLength as Int
        with(terminalInstance.terminal) {
            return when {
                version.isNullOrBlank() -> Validation.Blank.also {
                    println(red(Errors.blankInput(PromptType.PackageVersion)))
                }
                version.length > packageVersionMaxLength -> Validation.InvalidLength.also {
                    println(red(Errors.invalidLength(max = packageVersionMaxLength)))
                }
                !version.matches(Pattern.packageVersion) -> Validation.InvalidPattern.also {
                    println(red(Errors.invalidRegex(Pattern.packageVersion)))
                }
                else -> Validation.Success
            }
        }
    }

    suspend fun isInstallerUrlValid(
        url: String?,
        installerSchemaObj: InstallerSchema? = installerSchema,
        responseCallback: suspend () -> HttpResponse?
    ): Validation {
        val urlMaxLength = installerSchemaObj?.definitions?.installer?.properties?.installerUrl?.maxLength as Int
        with(terminalInstance.terminal) {
            return when {
                url.isNullOrBlank() -> Validation.Blank.also {
                    println(red(Errors.blankInput(PromptType.InstallerUrl)))
                }
                url.length > urlMaxLength -> Validation.InvalidLength.also {
                    println(red(Errors.invalidLength(max = urlMaxLength)))
                }
                !url.matches(Pattern.installerUrl) -> Validation.InvalidPattern.also {
                    println(red(Errors.invalidRegex(Pattern.installerUrl)))
                }
                else -> {
                    val installerUrlResponse: HttpResponse? = responseCallback()
                    val status = installerUrlResponse?.status ?: HttpStatusCode.BadRequest
                    if (!status.isSuccess() && !status.isRedirect()) {
                        println(red(Errors.unsuccessfulUrlResponse(installerUrlResponse)))
                        Validation.UnsuccessfulResponseCode
                    } else {
                        Validation.Success
                    }
                }
            }
        }
    }

    fun isArchitectureValid(architecture: String?): Validation {
        val architecturesEnum = installerSchema?.definitions?.architecture?.enum as List<String>
        with(terminalInstance.terminal) {
            return when {
                architecture.isNullOrBlank() -> Validation.Blank.also {
                    println(red(Errors.blankInput(PromptType.Architecture)))
                }
                !architecturesEnum.contains(architecture) -> Validation.InvalidArchitecture.also {
                    println(red(Errors.invalidEnum(Validation.InvalidArchitecture, architecturesEnum)))
                }
                else -> Validation.Success
            }
        }
    }

    fun isInstallerTypeValid(installerType: String?): Validation {
        val installerTypesEnum = installerSchema?.definitions?.installerType?.enum as List<String>
        with(terminalInstance.terminal) {
            return when {
                installerType.isNullOrBlank() -> Validation.Blank.also {
                    println(red(Errors.blankInput(PromptType.InstallerType)))
                }
                !installerTypesEnum.contains(installerType) -> Validation.InvalidInstallerType.also {
                    println(red(Errors.invalidEnum(Validation.InvalidInstallerType, installerTypesEnum)))
                }
                else -> Validation.Success
            }
        }
    }

    fun isSwitchValid(switch: String?, installerSwitch: InstallerSwitch, canBeBlank: Boolean): Validation {
        with(terminalInstance.terminal) {
            return when {
                switch.isNullOrBlank() && !canBeBlank -> Validation.Blank.also {
                    println(red(Errors.blankInput(getPromptTypeFromInstallerSwitch(installerSwitch))))
                }
                (switch?.length ?: 0) > getInstallerSwitchLengthBoundary(installerSwitch).second -> {
                    Validation.InvalidLength.also {
                        println(
                            red(
                                Errors.invalidLength(
                                    min = getInstallerSwitchLengthBoundary(installerSwitch).first,
                                    max = getInstallerSwitchLengthBoundary(installerSwitch).second
                                )
                            )
                        )
                    }
                }
                else -> Validation.Success
            }
        }
    }

    fun isInstallerLocaleValid(locale: String?): Validation {
        val installerLocaleMaxLength = installerSchema?.definitions?.locale?.maxLength as Int
        with(terminalInstance.terminal) {
            return when {
                !locale.isNullOrBlank() && !locale.matches(Pattern.installerLocale) -> Validation.InvalidPattern.also {
                    println(red(Errors.invalidRegex(Pattern.installerLocale)))
                }
                (locale?.length ?: 0) > installerLocaleMaxLength -> Validation.InvalidLength.also {
                    println(red(Errors.invalidLength(max = installerLocaleMaxLength)))
                }
                else -> Validation.Success
            }
        }
    }

    fun isProductCodeValid(productCode: String?): Validation {
        val productCodeMinLength = installerSchema?.definitions?.productCode?.minLength as Int
        val productCodeMaxLength = installerSchema?.definitions?.productCode?.maxLength as Int
        with(terminalInstance.terminal) {
            return when {
                !productCode.isNullOrBlank() && productCode.length > productCodeMaxLength -> {
                    Validation.InvalidLength.also {
                        println(red(Errors.invalidLength(min = productCodeMinLength, max = productCodeMaxLength)))
                    }
                }
                else -> Validation.Success
            }
        }
    }

    fun isInstallerScopeValid(option: Char?): Validation {
        val installerScopeEnum = installerSchema?.definitions?.scope?.enum as List<String>
        with(terminalInstance.terminal) {
            return when {
                option != Prompts.noIdea.first() && installerScopeEnum.all {
                    it.first().titlecase() != option?.titlecase()
                } -> Validation.InvalidInstallerScope.also {
                    println(red(Errors.invalidEnum(Validation.InvalidInstallerScope, installerScopeEnum)))
                }
                else -> Validation.Success
            }
        }
    }

    fun isUpgradeBehaviourValid(option: Char?): Validation {
        with(terminalInstance.terminal) {
            return when {
                upgradeBehaviourEnum.all {
                    it.first().titlecase() != option?.titlecase()
                } -> Validation.InvalidUpgradeBehaviour.also {
                    println(red(Errors.invalidEnum(Validation.InvalidUpgradeBehaviour, upgradeBehaviourEnum)))
                }
                else -> Validation.Success
            }
        }
    }

    fun isReleaseDateValid(releaseDate: String?): Validation {
        if (!releaseDate.isNullOrBlank()) {
            try {
                LocalDate.parse(releaseDate, DateTimeFormatter.ofPattern(Pattern.releaseDate, Locale.getDefault()))
            } catch (dateTimeParseException: DateTimeParseException) {
                terminalInstance.terminal.println(red(Errors.invalidReleaseDate(dateTimeParseException)))
                return Validation.InvalidReleaseDate
            }
        }
        return Validation.Success
    }

    private fun getInstallerSwitchLengthBoundary(installerSwitch: InstallerSwitch): Pair<Int, Int> {
        val installerSwitchProperties = installerSchema?.definitions?.installerSwitches?.properties
        return when (installerSwitch) {
            InstallerSwitch.Silent -> Pair(
                installerSwitchProperties?.silent?.minLength as Int,
                installerSwitchProperties.silent.maxLength
            )
            InstallerSwitch.SilentWithProgress -> Pair(
                installerSwitchProperties?.silentWithProgress?.minLength as Int,
                installerSwitchProperties.silentWithProgress.maxLength
            )
            InstallerSwitch.Custom -> Pair(
                installerSwitchProperties?.custom?.minLength as Int,
                installerSwitchProperties.custom.maxLength
            )
        }
    }

    private fun getPromptTypeFromInstallerSwitch(installerSwitch: InstallerSwitch): PromptType {
        return when (installerSwitch) {
            InstallerSwitch.Silent -> PromptType.SilentSwitch
            InstallerSwitch.SilentWithProgress -> PromptType.SilentWithProgressSwitch
            InstallerSwitch.Custom -> PromptType.CustomSwitch
        }
    }

    val architecturesEnum
        get() = installerSchema?.definitions?.architecture?.enum as List<String>

    val installerTypesEnum
        get() = installerSchema?.definitions?.installerType?.enum as List<String>

    val installerScopeEnum
        get() = installerSchema?.definitions?.scope?.enum as List<String>

    val upgradeBehaviourEnum
        get() = installerSchema?.definitions?.upgradeBehavior?.enum as List<String>

    companion object {
        const val packageIdentifierMinLength = 4
    }
}
