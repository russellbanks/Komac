package data

import com.charleskorn.kaml.SingleLineStringStyle
import com.charleskorn.kaml.Yaml
import com.charleskorn.kaml.YamlConfiguration
import kotlinx.serialization.modules.SerializersModule
import schemas.LocalDateSerializer
import java.time.LocalDate

object YamlConfig {
    private val sharedConfiguration = YamlConfiguration(
        encodeDefaults = false,
        singleLineStringStyle = SingleLineStringStyle.Plain
    )

    val defaultWithLocalDataSerializer = Yaml(
        serializersModule = SerializersModule {
            contextual(LocalDate::class, LocalDateSerializer)
        },
        configuration = sharedConfiguration
    )

    val default = Yaml(configuration = sharedConfiguration)
}
