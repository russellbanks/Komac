
import com.github.ajalt.clikt.core.CliktError
import data.GitHubImpl
import io.ktor.client.statement.HttpResponse

object Errors {
    const val error = "[Error]"
    const val connectionTimeout = "$error Connection timed out"
    const val connectionFailure = "$error Failed to connect"

    fun invalidLength(min: Number? = null, max: Number? = null, items: Iterable<String>? = null): String {
        return buildString {
            append("$error Invalid Length")
            if (min != null || max != null) {
                append(" -${items?.let { "Item" }.orEmpty()} ${items?.let { "Length" } ?: "length"} must be ")
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
            append("$error Invalid Pattern")
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
            append("$error Unsuccessful Response Code - The server ")
            append(response?.let { "responded with ${it.status}" } ?: "did not return a successful response")
        }
    }

    fun blankInput(promptName: String? = null) = "$error ${promptName ?: "Input"} cannot be blank"

    fun invalidEnum(enum: List<String>): String {
        return buildString {
            append(error)
            append(" - Value must exist in the enum - ")
            append(enum.joinToString(", "))
        }
    }

    fun doesNotExistError(packageIdentifier: String, packageVersion: String, isUpdate: Boolean = false): CliktError {
        return CliktError(
            message = buildString {
                appendLine("$packageIdentifier $packageVersion does not exist in ${GitHubImpl.wingetPkgsFullName}")
                if (isUpdate) appendLine("Please use the 'new' command to create a new manifest.")
            },
            statusCode = 1
        )
    }
}
