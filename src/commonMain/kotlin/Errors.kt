
import com.github.ajalt.clikt.core.CliktError
import com.github.ajalt.mordant.rendering.Theme
import github.WingetPkgs
import io.ktor.client.statement.HttpResponse

object Errors {
    const val connectionTimeout = "Connection timed out"
    const val connectionFailure = "Failed to connect"
    val noManifestChanges = """
        No changes were detected between the previous and new manifests.
        Please double-check if there were any changes made in the manifest, by comparing the manifests displayed here with the manifests already present in winget-pkgs.
    """.trimIndent()

    fun invalidLength(min: Number? = null, max: Number? = null, items: Iterable<String>? = null): String {
        return buildString {
            if (min != null || max != null) {
                append("${items?.let { "Item " }.orEmpty()}Length must be ")
            }
            when {
                min != null && max != null -> append("between $min and $max")
                min != null -> append("greater than $min")
                max != null -> append("less than $max")
            }
            append(" characters")
            items?.let { nonNullItems ->
                appendLine()
                appendLine("Items that did not match:")
                for (item in nonNullItems) {
                    appendLine(" - $item")
                }
            }
        }
    }

    fun invalidRegex(regex: Regex? = null, items: Iterable<String>? = null): String {
        return buildString {
            append("Invalid Pattern")
            regex?.let { append(" - Must match regex: $it") }
            items?.let { nonNullItems ->
                appendLine()
                appendLine("Items that did not match:")
                for (item in nonNullItems) {
                    appendLine(" - $item")
                }
            }
        }
    }

    fun unsuccessfulUrlResponse(response: HttpResponse?): String {
        return buildString {
            append("Unsuccessful Response Code - The server ")
            append(response?.let { "responded with ${it.status}" } ?: "did not return a successful response")
        }
    }

    fun blankInput(promptName: String? = null) = "${promptName ?: "Input"} cannot be blank"

    fun doesNotExistError(
        packageIdentifier: String,
        packageVersion: String? = null,
        isUpdate: Boolean = false,
        theme: Theme
    ): CliktError = CliktError(
        message = theme.warning(
            buildString {
                append("$packageIdentifier ")
                if (packageVersion != null) append("$packageVersion ")
                appendLine("does not exist in ${WingetPkgs.FULL_NAME}")
                if (isUpdate) appendLine("Please use the 'new' command to create a new manifest.")
            }
        ),
        statusCode = 1
    )
}
