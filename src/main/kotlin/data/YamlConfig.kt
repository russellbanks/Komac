package data

import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import kotlinx.serialization.modules.SerializersModule
import schemas.LocalDateSerializer
import java.time.LocalDate

object YamlConfig {
    val installer = Yaml(
        serializersModule = SerializersModule {
            contextual(LocalDate::class, LocalDateSerializer)
        },
        configuration = YamlConfiguration(
            encodeDefaults = false,
            singleLineStringStyle = SingleLineStringStyle.Plain
        )
    )

    val other = Yaml(
        configuration = YamlConfiguration(
            encodeDefaults = false,
            singleLineStringStyle = SingleLineStringStyle.Plain
        )
    )
}
