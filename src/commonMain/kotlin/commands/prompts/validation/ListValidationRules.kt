package commands.prompts.validation

data class ListValidationRules<T>(
    val maxItems: Int,
    val minItemLength: Int? = null,
    val maxItemLength: Int? = null,
    val regex: Regex? = null,
    val transform: ((String) -> List<T>),
    val additionalValidation: ((List<T>) -> String?)? = null
)
