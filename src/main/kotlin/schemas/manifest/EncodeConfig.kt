package schemas.manifest

import com.charleskorn.kaml.AmbiguousQuoteStyle
import com.charleskorn.kaml.MultiLineStringStyle
import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import io.ktor.http.Url
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNamingStrategy
import kotlinx.serialization.modules.SerializersModule
import schemas.manifest.serializers.JsonUrlSerializer
import schemas.manifest.serializers.YamlUrlSerializer

object EncodeConfig {
    val yamlDefault = Yaml(
        serializersModule = SerializersModule {
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

    @OptIn(ExperimentalSerializationApi::class)
    val jsonDefault = Json {
        namingStrategy = JsonNamingStrategy { _, _, serialName ->
            serialName.split(Regex("[^a-zA-Z0-9]+")).joinToString("") { it.replaceFirstChar(Char::titlecaseChar) }
        }
        isLenient = true
        ignoreUnknownKeys = true
        serializersModule = SerializersModule {
            contextual(Url::class, JsonUrlSerializer)
        }
    }
}
