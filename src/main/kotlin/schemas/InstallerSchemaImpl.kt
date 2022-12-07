package schemas

import Errors
import InstallerSwitch
import Ktor.isRedirect
import PromptType
import Validation
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.red
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

@Single
class InstallerSchemaImpl : KoinComponent {
    private val retrieveSchemas: RetrieveSchemas = get()
    private val client: HttpClient = retrieveSchemas.client
    private var installerSchema: InstallerSchema? = null
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

    suspend fun isPackageIdentifierValid(identifier: String?): Validation {
        awaitInstallerSchema()
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
                !identifier.matches(packageIdentifierPattern) -> Validation.InvalidPattern.also {
                    println(red(Errors.invalidRegex(packageIdentifierPattern)))
                }
                else -> Validation.Success
            }
        }
    }

    fun isPackageVersionValid(version: String?): Validation {
        with(terminalInstance.terminal) {
            return when {
                version.isNullOrBlank() -> Validation.Blank.also {
                    println(red(Errors.blankInput(PromptType.PackageVersion)))
                }
                version.length > packageVersionMaxLength -> Validation.InvalidLength.also {
                    println(red(Errors.invalidLength(max = packageVersionMaxLength)))
                }
                !version.matches(packageVersionPattern) -> Validation.InvalidPattern.also {
                    println(red(Errors.invalidRegex(packageVersionPattern)))
                }
                else -> Validation.Success
            }
        }
    }

    suspend fun isInstallerUrlValid(url: String?, responseCallback: suspend () -> HttpResponse?): Validation {
        with(terminalInstance.terminal) {
            return when {
                url.isNullOrBlank() -> Validation.Blank.also {
                    println(red(Errors.blankInput(PromptType.InstallerUrl)))
                }
                url.length > installerUrlMaxLength -> Validation.InvalidLength.also {
                    println(red(Errors.invalidLength(max = installerUrlMaxLength)))
                }
                !url.matches(installerUrlPattern) -> Validation.InvalidPattern.also {
                    println(red(Errors.invalidRegex(installerUrlPattern)))
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
        with(terminalInstance.terminal) {
            return when {
                architecture.isNullOrBlank() -> Validation.Blank.also {
                    println(red(Errors.blankInput(PromptType.Architecture)))
                }
                !architecturesEnum.contains(architecture) -> Validation.InvalidArchitecture.also {
                    println(red(Errors.invalidEnum(Validation.InvalidArchitecture, this@InstallerSchemaImpl)))
                }
                else -> Validation.Success
            }
        }
    }

    fun isInstallerTypeValid(installerType: String?): Validation {
        with(terminalInstance.terminal) {
            return when {
                installerType.isNullOrBlank() -> Validation.Blank.also {
                    println(red(Errors.blankInput(PromptType.InstallerType)))
                }
                !installerTypesEnum.contains(installerType) -> Validation.InvalidInstallerType.also {
                    println(red(Errors.invalidEnum(Validation.InvalidInstallerType, this@InstallerSchemaImpl)))
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

    private val packageIdentifierPattern
        get() = installerSchema?.definitions?.packageIdentifier?.pattern?.toRegex() as Regex

    private val packageIdentifierMaxLength
        get() = installerSchema?.definitions?.packageIdentifier?.maxLength as Int

    private val packageVersionPattern
        get() = installerSchema?.definitions?.packageVersion?.pattern?.toRegex() as Regex

    private val packageVersionMaxLength
        get() = installerSchema?.definitions?.packageVersion?.maxLength as Int

    private val installerUrlPattern
        get() = installerSchema?.definitions?.installer?.properties?.installerUrl?.pattern?.toRegex() as Regex

    private val installerUrlMaxLength
        get() = installerSchema?.definitions?.installer?.properties?.installerUrl?.maxLength as Int

    val architecturesEnum
        get() = installerSchema?.definitions?.architecture?.enum as List<String>

    val installerTypesEnum
        get() = installerSchema?.definitions?.installerType?.enum as List<String>

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

    companion object {
        const val packageIdentifierMinLength = 4
    }
}
