package data

import Errors
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

    private const val packageIdentifierMinLength = 4
}
