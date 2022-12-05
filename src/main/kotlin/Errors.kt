import io.ktor.client.statement.HttpResponse
import schemas.InstallerSchemaImpl

object Errors {
    private const val error = "[Error]"

    fun invalidLength(min: Int? = null, max: Int? = null): String {
        return buildString {
            append("$error ${Validation.InvalidLength}")
            if (min != null || max != null) append(" - Length must be ")
            when {
                min != null && max != null -> append("between $min and $max")
                min != null -> append("greater than $min")
                max != null -> append("less than $max")
            }
        }
    }

    fun invalidRegex(regex: Regex? = null): String {
        return buildString {
            append("$error ${Validation.InvalidPattern}")
            regex?.let { append(" - Must match regex: $it") }
        }
    }

    fun unsuccessfulUrlResponse(response: HttpResponse?): String {
        return buildString {
            append("$error ${Validation.UnsuccessfulResponseCode} - The server ")
            append(response?.let { "responded with ${response.status}" } ?: "did not return a successful response")
        }
    }

    fun blankInput(promptType: PromptType? = null) = "$error ${promptType ?: "Input"} cannot be blank"

    fun invalidEnum(validation: Validation, installerSchemaImpl: InstallerSchemaImpl): String {
        return buildString {
            append("$error $validation - Value must exist in the enum - ")
            append(installerSchemaImpl.architecturesEnum.joinToString(", "))
        }
    }
}
