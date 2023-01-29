import data.locale.DescriptionType
import input.LocaleType
import io.ktor.client.statement.HttpResponse

object Errors {
    const val error = "[Error]"

    fun invalidLength(min: Number? = null, max: Number? = null, items: Iterable<String>? = null): String {
        return buildString {
            append("$error ${Validation.InvalidLength}")
            if (min != null || max != null) {
                append(" -${items?.let { "Item" } ?: ""} ${items?.let { "Length" } ?: "length"} must be ")
            }
            when {
                min != null && max != null -> append("between $min and $max")
                min != null -> append("greater than $min")
                max != null -> append("less than $max")
            }
            items?.let { nonNullItems ->
                appendLine()
                appendLine("Items that did not match:")
                nonNullItems.forEach {
                    appendLine(" - $it")
                }
            }
        }
    }

    fun invalidRegex(regex: Regex? = null, items: Iterable<String>? = null): String {
        return buildString {
            append("$error ${Validation.InvalidPattern}")
            regex?.let { append(" - Must match regex: $it") }
            items?.let { nonNullItems ->
                appendLine()
                appendLine("Items that did not match:")
                nonNullItems.forEach {
                    appendLine(" - $it")
                }
            }
        }
    }

    fun unsuccessfulUrlResponse(response: HttpResponse?): String {
        return buildString {
            append("$error ${Validation.UnsuccessfulResponseCode} - The server ")
            append(response?.let { "responded with ${it.status}" } ?: "did not return a successful response")
        }
    }

    fun blankInput(descriptionType: DescriptionType? = null) = blankInput(descriptionType?.promptName)

    fun blankInput(localeType: LocaleType? = null) = blankInput(localeType.toString())

    fun blankInput(promptName: String? = null) = "$error ${promptName ?: "Input"} cannot be blank"

    fun invalidEnum(validation: Validation?, enum: List<String>): String {
        return buildString {
            append(error)
            if (validation != null) append(" $validation")
            append(" - Value must exist in the enum - ")
            append(enum.joinToString(", "))
        }
    }

    const val connectionTimeout = "$error Connection timed out"
    const val connectionFailure = "$error Failed to connect"
}
