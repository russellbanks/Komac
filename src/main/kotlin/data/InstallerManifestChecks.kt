package data

import Errors
import InstallerSwitch
import Ktor.isRedirect
import Validation
import input.PromptType
import io.ktor.client.HttpClient
import io.ktor.client.engine.java.Java
import io.ktor.client.plugins.UserAgent
import io.ktor.client.request.head
import io.ktor.client.statement.HttpResponse
import io.ktor.http.isSuccess
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.InstallerSchema
import schemas.InstallerSchemaImpl
import schemas.Pattern
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException
import java.util.Locale

object InstallerManifestChecks : KoinComponent {

    fun isPackageIdentifierValid(
        identifier: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val packageIdentifierMaxLength = installerSchema.definitions.packageIdentifier.maxLength
        val packageIdentifierRegex = Pattern.packageIdentifier(installerSchema)
        return when {
            identifier.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.PackageVersion)
            identifier.length > packageIdentifierMaxLength -> Validation.InvalidLength to Errors.invalidLength(
                min = packageIdentifierMinLength,
                max = packageIdentifierMaxLength
            )
            !identifier.matches(packageIdentifierRegex) -> {
                Validation.InvalidPattern to Errors.invalidRegex(packageIdentifierRegex)
            }
            else -> Validation.Success to null
        }
    }

    fun isPackageVersionValid(
        version: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val packageVersionMaxLength = installerSchema.definitions.packageVersion.maxLength
        val packageVersionRegex = Pattern.packageVersion(installerSchema)
        return when {
            version.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.PackageVersion)
            version.length > packageVersionMaxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = packageVersionMaxLength)
            }
            !version.matches(packageVersionRegex) -> {
                Validation.InvalidPattern to Errors.invalidRegex(packageVersionRegex)
            }
            else -> Validation.Success to null
        }
    }

    suspend fun isInstallerUrlValid(
        url: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerUrlMaxLength = installerSchema.definitions.installer.properties.installerUrl.maxLength
        val installerUrlRegex = Pattern.installerUrl(installerSchema)
        return when {
            url.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.InstallerUrl)
            url.length > installerUrlMaxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = installerUrlMaxLength)
            }
            !url.matches(installerUrlRegex) -> Validation.InvalidPattern to Errors.invalidRegex(installerUrlRegex)
            else -> {
                lateinit var installerUrlResponse: HttpResponse
                HttpClient(Java) {
                    install(UserAgent) {
                        agent = "Microsoft-Delivery-Optimization/10.1"
                    }
                    followRedirects = false
                }.use { installerUrlResponse = it.head(url) }
                if (!installerUrlResponse.status.isSuccess() && !installerUrlResponse.status.isRedirect()) {
                    Validation.UnsuccessfulResponseCode to Errors.unsuccessfulUrlResponse(installerUrlResponse)
                } else {
                    Validation.Success to null
                }
            }
        }
    }

    fun isArchitectureValid(
        architecture: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val architecturesEnum = installerSchema.definitions.architecture.enum
        return when {
            architecture.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.Architecture)
            !architecturesEnum.contains(architecture) -> {
                Validation.InvalidArchitecture to Errors.invalidEnum(Validation.InvalidArchitecture, architecturesEnum)
            }
            else -> Validation.Success to null
        }
    }

    fun isInstallerTypeValid(
        installerType: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerTypesEnum = installerSchema.definitions.installerType.enum
        return when {
            installerType.isNullOrBlank() -> Validation.Blank to Errors.blankInput(PromptType.InstallerType)
            !installerTypesEnum.contains(installerType) -> {
                Validation.InvalidInstallerType to Errors.invalidEnum(
                    Validation.InvalidInstallerType,
                    installerTypesEnum
                )
            }
            else -> Validation.Success to null
        }
    }

    fun isInstallerSwitchValid(
        switch: String?,
        installerSwitch: InstallerSwitch,
        canBeBlank: Boolean = false,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val (minBoundary, maxBoundary) = installerSwitch.getLengthBoundary(installerSchema)
        return when {
            switch.isNullOrBlank() && !canBeBlank -> {
                Validation.Blank to Errors.blankInput(installerSwitch.toPromptType())
            }
            (switch?.length ?: 0) > maxBoundary -> {
                Validation.InvalidLength to Errors.invalidLength(min = minBoundary, max = maxBoundary)
            }
            else -> Validation.Success to null
        }
    }

    fun isInstallerLocaleValid(
        locale: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerLocaleMaxLength = installerSchema.definitions.locale.maxLength
        val installerLocaleRegex = Pattern.installerLocale(installerSchema)
        return when {
            !locale.isNullOrBlank() && !locale.matches(installerLocaleRegex) -> {
                Validation.InvalidPattern to Errors.invalidRegex(installerLocaleRegex)
            }
            (locale?.length ?: 0) > installerLocaleMaxLength -> {
                Validation.InvalidLength to Errors.invalidLength(max = installerLocaleMaxLength)
            }
            else -> Validation.Success to null
        }
    }

    fun isProductCodeValid(
        productCode: String?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val productCodeMinLength = installerSchema.definitions.productCode.minLength
        val productCodeMaxLength = installerSchema.definitions.productCode.maxLength
        return when {
            !productCode.isNullOrBlank() && productCode.length > productCodeMaxLength -> {
                Validation.InvalidLength to Errors.invalidLength(
                    min = productCodeMinLength,
                    max = productCodeMaxLength
                )
            }
            else -> Validation.Success to null
        }
    }

    fun isInstallerScopeValid(
        option: Char?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val installerScopeEnum = installerSchema.definitions.scope.enum
        return when {
            option != input.Prompts.noIdea.first() && installerScopeEnum.all {
                it.first().titlecase() != option?.titlecase()
            } -> Validation.InvalidInstallerScope to Errors.invalidEnum(
                Validation.InvalidInstallerScope,
                installerScopeEnum
            )
            else -> Validation.Success to null
        }
    }

    fun isUpgradeBehaviourValid(
        option: Char?,
        installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema
    ): Pair<Validation, String?> {
        val upgradeBehaviourEnum = installerSchema.definitions.upgradeBehavior.enum
        return when {
            upgradeBehaviourEnum.all {
                it.first().titlecase() != option?.titlecase()
            } -> Validation.InvalidUpgradeBehaviour to Errors.invalidEnum(
                Validation.InvalidUpgradeBehaviour,
                upgradeBehaviourEnum
            )
            else -> Validation.Success to null
        }
    }

    fun isReleaseDateValid(releaseDate: String?): Pair<Validation, String?> {
        if (!releaseDate.isNullOrBlank()) {
            try {
                LocalDate.parse(releaseDate, DateTimeFormatter.ofPattern(Pattern.releaseDate, Locale.getDefault()))
            } catch (dateTimeParseException: DateTimeParseException) {
                return Validation.InvalidReleaseDate to Errors.invalidReleaseDate(dateTimeParseException)
            }
        }
        return Validation.Success to null
    }

    private const val packageIdentifierMinLength = 4
}
