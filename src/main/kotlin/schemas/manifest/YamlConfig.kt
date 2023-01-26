package schemas.manifest

import com.charleskorn.kaml.MultiLineStringStyle
import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import io.ktor.http.Url
import kotlinx.serialization.modules.SerializersModule
import schemas.manifest.serializers.YamlLocalDateSerializer
import schemas.manifest.serializers.YamlUrlSerializer
import java.time.LocalDate

object YamlConfig {
    val default = Yaml(
        serializersModule = SerializersModule {
            contextual(LocalDate::class, YamlLocalDateSerializer)
            contextual(Url::class, YamlUrlSerializer)
        },
        configuration = YamlConfiguration(
            encodeDefaults = false,
            singleLineStringStyle = SingleLineStringStyle.Plain,
            multiLineStringStyle = MultiLineStringStyle.Literal
        )
    )
}
