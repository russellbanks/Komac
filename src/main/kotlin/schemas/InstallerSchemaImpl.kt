package schemas

import Errors
import Validation
import com.github.ajalt.mordant.animation.progressAnimation
import com.github.ajalt.mordant.rendering.TextColors.red
import input.Prompts
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.utils.io.core.use
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException
import java.util.Locale

@Single
class InstallerSchemaImpl : KoinComponent {
    lateinit var installerSchema: InstallerSchema
    private val terminalInstance: TerminalInstance by inject()

    private var asyncJob: Deferred<Unit> = CoroutineScope(Dispatchers.Default).async {
        val json = Json { ignoreUnknownKeys = true }
        HttpClient(Java) {
            install(ContentNegotiation)
            install(UserAgent) {
                agent = "Microsoft-Delivery-Optimization/10.1"
            }
        }.use {
            installerSchema = json.decodeFromString(it.get(Schemas.installerSchema).body())
        }
    }

    suspend fun awaitInstallerSchema() {
        with(asyncJob) {
            if (isActive) {
                terminalInstance.terminal.progressAnimation {
                    text("Retrieving installer schema")
                    progressBar()
                }.run {
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

    fun isInstallerLocaleValid(
        locale: String?,
        installerSchemaObj: InstallerSchema = installerSchema
    ): Validation {
        val installerLocaleMaxLength = installerSchemaObj.definitions.locale.maxLength
        with(terminalInstance.terminal) {
            return when {
                !locale.isNullOrBlank() && !locale.matches(Pattern.installerLocale()) -> Validation.InvalidPattern.also {
                    println(red(Errors.invalidRegex(Pattern.installerLocale())))
                }
                (locale?.length ?: 0) > installerLocaleMaxLength -> Validation.InvalidLength.also {
                    println(red(Errors.invalidLength(max = installerLocaleMaxLength)))
                }
                else -> Validation.Success
            }
        }
    }

    fun isProductCodeValid(
        productCode: String?,
        installerSchemaObj: InstallerSchema = installerSchema
    ): Validation {
        val productCodeMinLength = installerSchemaObj.definitions.productCode.minLength
        val productCodeMaxLength = installerSchemaObj.definitions.productCode.maxLength
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

    fun isInstallerScopeValid(
        option: Char?,
        installerSchemaObj: InstallerSchema = installerSchema
    ): Validation {
        val installerScopeEnum = installerSchemaObj.definitions.scope.enum
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

    val architecturesEnum
        get() = installerSchema.definitions.architecture.enum

    val installerTypesEnum
        get() = installerSchema.definitions.installerType.enum

    val installerScopeEnum
        get() = installerSchema.definitions.scope.enum

    val upgradeBehaviourEnum
        get() = installerSchema.definitions.upgradeBehavior.enum
}
