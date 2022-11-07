package schemas

import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.JsonContentPolymorphicSerializer
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

object ManifestSerializer : JsonContentPolymorphicSerializer<Schema>(Schema::class) {
    override fun selectDeserializer(element: JsonElement): KSerializer<out Schema> {
        val id = element.jsonObject["\$id"]?.jsonPrimitive?.content
        return when {
            id?.contains("version") == true -> VersionSchema.serializer()
            id?.contains("installer") == true -> InstallerSchema.serializer()
            else -> throw IllegalArgumentException("Unknown manifest type")
        }
    }
}