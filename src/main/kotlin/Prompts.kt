import schemas.InstallerSchemaImpl
import schemas.Schemas

object Prompts {
    private const val required = "[Required]"

    const val packageIdentifierInfo = "$required Enter the Package Identifier, in the following format " +
        "<Publisher shortname.Application shortname>. For example: Microsoft.Excel"

    const val packageIdentifier = "Package Identifier"

    const val packageVersionInfo = "$required Enter the version. For example: 1.33.7"

    const val packageVersion = "Package Version"

    const val installerUrlInfo = "$required Enter the download url to the installer."

    const val installerUrl = "Url"

    fun architectureInfo(installerSchemaImpl: InstallerSchemaImpl): String {
        return "$required Enter the architecture. Options: ${installerSchemaImpl.architecturesEnum.joinToString(", ")}"
    }

    const val architecture = "Architecture"

    fun installerTypeInfo(installerSchemaImpl: InstallerSchemaImpl): String {
        return buildString {
            append("$required Enter the installer type. Options: ")
            append(installerSchemaImpl.installerTypesEnum.joinToString(", "))
        }
    }

    fun silentSwitchInfo(installerType: String?): String {
        return buildString {
            append(if (installerType == Schemas.InstallerType.exe) "[Required]" else "[Optional]")
            append(" Enter the silent install switch. For example: /S, -verysilent, /qn, --silent, /exenoui")
        }
    }

    const val silentSwitch = "Silent Switch"

    const val installerType = "Installer Type"

    object Redirection {
        fun originalUrlRetained(url: String?) = "Original URL Retained - Proceeding with $url"

        fun discoveredUrl(url: String?) = "Discovered URL: $url"

        const val redirectFound = "The URL appears to be redirected. Would you like to use the destination URL instead?"

        const val useDetectedUrl = "   [Y] Use detected URL"

        const val detectedUrlValidationFailed = "Validation has failed for the detected URL. Using original URL."

        const val useOriginalUrl = "   [N] Use original URL"

        const val enterChoice = "Enter Choice"

        const val urlChanged = "[Warning] URL Changed - The URL was changed during processing and will be re-validated"
    }
}
