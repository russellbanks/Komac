package schemas

import Ktor.isRedirect
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
        return when {
            identifier?.length !in packageIdentifierMinLength until packageIdentifierMaxLength -> {
                Validation.InvalidLength
            }
            packageIdentifierPattern?.let { identifier?.matches(it) } != true -> Validation.InvalidPattern
            else -> Validation.Success
        }
    }

    fun isPackageVersionValid(version: String?): Validation {
        return when {
            version.isNullOrBlank() -> Validation.Blank
            version.length > packageVersionMaxLength -> Validation.InvalidLength
            !version.matches(packageVersionPattern) -> Validation.InvalidPattern
            else -> Validation.Success
        }
    }

    suspend fun isInstallerUrlValid(url: String?, responseCallback: suspend () -> HttpResponse?): Validation {
        return when {
            url.isNullOrBlank() -> Validation.Blank
            url.length > installerUrlMaxLength -> Validation.InvalidLength
            !url.matches(installerUrlPattern) -> Validation.InvalidPattern
            else -> {
                val status = responseCallback()?.status ?: HttpStatusCode.BadRequest
                if (status.isSuccess() || status.isRedirect()) {
                    Validation.Success
                } else {
                    Validation.UnsuccessfulResponseCode
                }
            }
        }
    }

    fun isArchitectureValid(architecture: String?): Validation {
        return when {
            architecture.isNullOrBlank() -> Validation.Blank
            installerSchema?.definitions?.architecture?.enum?.contains(architecture) != true -> {
                Validation.InvalidArchitecture
            }
            else -> Validation.Success
        }
    }

    fun isInstallerTypeValid(installerType: String?): Validation {
        return when {
            installerType.isNullOrBlank() -> Validation.Blank
            installerSchema?.definitions?.installerType?.enum?.contains(installerType) != true -> {
                Validation.InvalidInstallerType
            }
            else -> Validation.Success
        }
    }

    fun isSilentSwitchValid(silentSwitch: String?, canBeBlank: Boolean): Validation {
        return when {
            silentSwitch.isNullOrBlank() && !canBeBlank -> {
                terminalInstance.terminal.println(red(Errors.blankInput(PromptType.SilentSwitch)))
                Validation.Blank
            }
            (silentSwitch?.length ?: 0) > installerSilentSwitchMaxLength -> {
                terminalInstance.terminal.println(
                    red(
                        Errors.invalidLength(min = installerSilentSwitchMinLength, max = installerSilentSwitchMaxLength)
                    )
                )
                Validation.InvalidLength
            }
            else -> Validation.Success
        }
    }

    val packageIdentifierPattern
        get() = installerSchema?.definitions?.packageIdentifier?.pattern?.toRegex()

    val packageIdentifierMaxLength
        get() = installerSchema?.definitions?.packageIdentifier?.maxLength as Int

    val packageVersionPattern
        get() = installerSchema?.definitions?.packageVersion?.pattern?.toRegex() as Regex

    val packageVersionMaxLength
        get() = installerSchema?.definitions?.packageVersion?.maxLength as Int

    val installerUrlPattern
        get() = installerSchema?.definitions?.installer?.properties?.installerUrl?.pattern?.toRegex() as Regex

    val installerUrlMaxLength
        get() = installerSchema?.definitions?.installer?.properties?.installerUrl?.maxLength as Int

    val architecturesEnum
        get() = installerSchema?.definitions?.architecture?.enum as List<String>

    val installerTypesEnum
        get() = installerSchema?.definitions?.installerType?.enum as List<String>

    private val installerSilentSwitchMinLength
        get() = installerSchema?.definitions?.installerSwitches?.properties?.silent?.minLength as Int

    private val installerSilentSwitchMaxLength
        get() = installerSchema?.definitions?.installerSwitches?.properties?.silent?.maxLength as Int

    companion object {
        const val packageIdentifierMinLength = 4
    }
}
