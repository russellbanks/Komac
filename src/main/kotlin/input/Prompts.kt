package input

import InstallerSwitch
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.Enum
import schemas.InstallerSchema
import schemas.InstallerSchemaImpl
import schemas.Schemas

object Prompts : KoinComponent {
    private const val required = "[Required]"
    private const val optional = "[Optional]"

    const val optionIndent = 3

    const val packageIdentifierInfo = "$required Enter the Package Identifier, in the following format " +
        "<Publisher shortname.Application shortname>. For example: Microsoft.Excel"

    const val packageIdentifier = "Package Identifier"

    const val packageVersionInfo = "$required Enter the version. For example: 1.33.7"

    const val packageVersion = "Package Version"

    const val installerUrlInfo = "$required Enter the download url to the installer."

    const val installerLocaleInfo = "$optional Enter the installer locale. For example: en-US, en-CA"

    const val productCodeInfo = "$optional Enter the application product code. " +
        "Looks like {CF8E6E00-9C03-4440-81C0-21FACB921A6B}"

    const val installerScopeInfo = "$optional Enter the Installer Scope"

    const val enterChoice = "Enter Choice"

    const val noIdea = "No idea"

    const val upgradeBehaviourInfo = "$optional Enter the Upgrade Behavior"

    const val releaseDateInfo = "$optional Enter the application release date in the format YYYY-MM-DD. " +
        "Example: 2022-11-17"

    const val additionalInstallerInfo = "Do you want to create another installer?"

    fun architectureInfo(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): String {
        return "$required Enter the architecture. Options: ${Enum.architecture(installerSchema).joinToString(", ")}"
    }

    fun installerTypeInfo(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): String {
        return buildString {
            append("$required Enter the installer type. Options: ")
            append(Enum.installerType(installerSchema).joinToString(", "))
        }
    }

    fun switchInfo(installerType: String?, installerSwitch: InstallerSwitch): String {
        return buildString {
            append(
                when {
                    installerType == Schemas.InstallerType.exe && installerSwitch != InstallerSwitch.Custom -> required
                    else -> optional
                }
            )
            append(" Enter the ${installerSwitch.toString().lowercase()}. For example: ")
            append(
                when (installerSwitch) {
                    InstallerSwitch.Silent -> "/S, -verysilent, /qn, --silent, /exenoui."
                    InstallerSwitch.SilentWithProgress -> "/S, -silent, /qb, /exebasicui."
                    InstallerSwitch.Custom -> "/norestart, -norestart"
                }
            )
        }
    }

    fun fileExtensionsInfo(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): String {
        return buildString {
            append("$optional Enter any File Extensions the application could support. For example: html, htm, url")
            append(" (Max: ${installerSchema.definitions.fileExtensions.maxItems})")
        }
    }

    fun protocolsInfo(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): String {
        return buildString {
            append("$optional Enter any Protocols the application provides a handler for. For example: http, https")
            append(" (Max: ${installerSchema.definitions.protocols.maxItems})")
        }
    }

    const val installerType = "Installer Type"

    object Redirection {
        fun originalUrlRetained(url: String?) = "Original URL Retained - Proceeding with $url"

        fun discoveredUrl(url: String?) = "Discovered URL: $url"

        const val redirectFound = "The URL appears to be redirected. Would you like to use the destination URL instead?"

        const val useDetectedUrl = "   [Y] Use detected URL"

        const val detectedUrlValidationFailed = "Validation has failed for the detected URL. Using original URL."

        const val useOriginalUrl = "   [N] Use original URL"

        const val urlChanged = "[Warning] URL Changed - The URL was changed during processing and will be re-validated"
    }
}
