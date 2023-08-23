package commands.prompts.validation

data class ValidationRules(
    val maxLength: Int? = null,
    val minLength: Int? = null,
    val pattern: Regex? = null,
    val isRequired: Boolean = true
)
