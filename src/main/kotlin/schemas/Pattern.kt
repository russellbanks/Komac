package schemas

import org.koin.core.component.KoinComponent
import org.koin.core.component.get

object Pattern : KoinComponent {
    fun packageIdentifier(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): Regex {
        return Regex(installerSchema.definitions.packageIdentifier.pattern)
    }

    fun packageVersion(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): Regex {
        return Regex(installerSchema.definitions.packageVersion.pattern)
    }

    fun installerUrl(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): Regex {
        return Regex(installerSchema.definitions.installer.properties.installerUrl.pattern)
    }

    fun installerLocale(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): Regex {
        return Regex(installerSchema.definitions.locale.pattern)
    }

    const val releaseDate = "yyyy-MM-dd"
}
