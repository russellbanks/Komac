import schemas.ManifestVersionSchema

class Patterns(private val manifestVersionSchema: ManifestVersionSchema) {
    val packageIdentifier = manifestVersionSchema.properties.packageIdentifier.pattern.toRegex()
    val packageIdentifierMaxLength = manifestVersionSchema.properties.packageIdentifier.maxLength
    val packageVersion = manifestVersionSchema.properties.packageVersion.pattern.toRegex()
}