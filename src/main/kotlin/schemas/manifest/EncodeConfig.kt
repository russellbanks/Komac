package schemas.manifest

import com.charleskorn.kaml.AmbiguousQuoteStyle
import com.charleskorn.kaml.MultiLineStringStyle
import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import io.ktor.http.Url
import kotlinx.serialization.json.Json
import kotlinx.serialization.modules.SerializersModule
import schemas.manifest.serializers.JsonLocalDateSerializer
import schemas.manifest.serializers.JsonUrlSerializer
import schemas.manifest.serializers.YamlLocalDateSerializer
import schemas.manifest.serializers.YamlUrlSerializer
import java.time.LocalDate

object EncodeConfig {
    val yamlDefault = Yaml(
        serializersModule = SerializersModule {
            contextual(LocalDate::class, YamlLocalDateSerializer)
            contextual(Url::class, YamlUrlSerializer)
        },
        configuration = YamlConfiguration(
            encodeDefaults = false,
            singleLineStringStyle = SingleLineStringStyle.PlainExceptAmbiguous,
            multiLineStringStyle = MultiLineStringStyle.Literal,
            breakScalarsAt = Int.MAX_VALUE,
            ambiguousQuoteStyle = AmbiguousQuoteStyle.SingleQuoted
        )
    )
    val jsonDefault = Json {
        serializersModule = SerializersModule {
            contextual(LocalDate::class, JsonLocalDateSerializer)
            contextual(Url::class, JsonUrlSerializer)
        }
    }
}
