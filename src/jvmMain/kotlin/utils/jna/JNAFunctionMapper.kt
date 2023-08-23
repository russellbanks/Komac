package utils.jna

import com.charleskorn.kaml.YamlNamingStrategy
import com.sun.jna.FunctionMapper

object JNAFunctionMapper {
    val snakeCaseMapper = FunctionMapper { _, method ->
        YamlNamingStrategy.SnakeCase.serialNameForYaml(method.name)
    }
}
