package schemas

import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

object Pattern : KoinComponent {
    private val installerSchemaImpl: InstallerSchemaImpl by inject()
    private val installerSchema = installerSchemaImpl.installerSchema

    val packageIdentifier = installerSchema?.definitions?.packageIdentifier?.pattern?.toRegex() as Regex

    val packageVersion = installerSchema?.definitions?.packageVersion?.pattern?.toRegex() as Regex

    val installerUrl = installerSchema?.definitions?.installer?.properties?.installerUrl?.pattern?.toRegex() as Regex

    val installerLocale = installerSchema?.definitions?.locale?.pattern?.toRegex() as Regex

    const val releaseDate = "yyyy-MM-dd"
}