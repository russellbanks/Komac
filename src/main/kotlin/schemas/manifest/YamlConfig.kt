package schemas.manifest

import com.charleskorn.kaml.MultiLineStringStyle
import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import io.ktor.http.Url
import kotlinx.serialization.modules.SerializersModule
import schemas.manifest.serializers.LocalDateSerializer
import schemas.manifest.serializers.UrlSerializer
import java.time.LocalDate

object YamlConfig {
    val default = Yaml(
        serializersModule = SerializersModule {
            contextual(LocalDate::class, LocalDateSerializer)
            contextual(Url::class, UrlSerializer)
        },
        configuration = YamlConfiguration(
            encodeDefaults = false,
            singleLineStringStyle = SingleLineStringStyle.Plain,
            multiLineStringStyle = MultiLineStringStyle.Literal,
            breakScalarsAt = Int.MAX_VALUE
        )
    )
}
