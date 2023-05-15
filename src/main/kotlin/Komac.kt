
import com.github.ajalt.clikt.core.NoOpCliktCommand

class Komac : NoOpCliktCommand(printHelpOnEmptyArgs = true) {
    override fun aliases(): Map<String, List<String>> = mapOf(
        "up" to listOf("update"),
        "rm" to listOf("remove"),
        "delete" to listOf("remove")
    )
}
