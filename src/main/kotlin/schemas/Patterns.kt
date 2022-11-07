package schemas

class Patterns(schemas: List<Schema?>) {
    private val versionSchema = schemas.find { it is VersionSchema } as VersionSchema
    private val installerSchema = schemas.find { it is InstallerSchema } as InstallerSchema

    val packageIdentifier = versionSchema.properties.packageIdentifier.pattern.toRegex()
    val packageIdentifierMaxLength = versionSchema.properties.packageIdentifier.maxLength
    val packageVersion = versionSchema.properties.packageVersion.pattern.toRegex()

    val installerUrlMaxLength = installerSchema.definitions.installer.properties.installerUrl.maxLength

    companion object {
        const val packageIdentifierMinLength = 4
    }
}
