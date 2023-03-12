package schemas.manifest.serializers

import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import java.time.LocalDate

object YamlLocalDateSerializer : KSerializer<LocalDate> {
    override val descriptor = PrimitiveSerialDescriptor(LocalDate::class.simpleName as String, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: LocalDate) {
        encoder.encodeString(value.toString())
    }

    override fun deserialize(decoder: Decoder): LocalDate {
        return LocalDate.parse(decoder.beginStructure(descriptor).decodeStringElement(descriptor, 0))
    }
}
