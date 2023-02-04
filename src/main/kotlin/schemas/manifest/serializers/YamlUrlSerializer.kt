package schemas.manifest.serializers

import io.ktor.http.Url
import io.ktor.http.decodeURLPart
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

object YamlUrlSerializer : KSerializer<Url> {
    override val descriptor = PrimitiveSerialDescriptor(Url::class.simpleName!!, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: Url) {
        encoder.encodeString(value.toString().removeSuffix("/").decodeURLPart())
    }

    override fun deserialize(decoder: Decoder): Url {
        return Url(decoder.beginStructure(descriptor).decodeStringElement(descriptor, 0).decodeURLPart())
    }
}
