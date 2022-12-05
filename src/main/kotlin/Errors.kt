import io.ktor.client.statement.HttpResponse
import schemas.InstallerSchemaImpl

object Errors {
    private const val error = "[Error]"

    fun invalidLength(min: Int? = null, max: Int? = null): String {
        return when {
            min != null && max != null -> "$error ${Validation.InvalidLength} - Length must be between $min and $max"
            min != null -> "$error ${Validation.InvalidLength} - Length must be greater than $min"
            max != null -> "$error ${Validation.InvalidLength} - Length must be less than $max"
            else -> "$error ${Validation.InvalidLength}"
        }
    }

    fun invalidRegex(regex: Regex? = null): String {
        return when {
            regex != null -> "$error ${Validation.InvalidPattern} - Must match regex: $regex"
            else -> "$error ${Validation.InvalidPattern}"
        }
    }

    fun unsuccessfulUrlResponse(response: HttpResponse?): String {
        return if (response != null) {
            "$error ${Validation.UnsuccessfulResponseCode} - The server responded with ${response.status}"
        } else {
            "$error ${Validation.UnsuccessfulResponseCode} - The server did not return a successful response"
        }
    }

    fun blankInput(promptType: PromptType? = null): String {
        return "$error ${promptType ?: "Input"} cannot be blank"
    }

    fun invalidEnum(validation: Validation, installerSchemaImpl: InstallerSchemaImpl): String {
        return "$error $validation - Value must exist in the enum - " +
                installerSchemaImpl.architecturesEnum.joinToString(", ")
    }
}
