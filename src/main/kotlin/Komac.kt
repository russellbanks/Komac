
import com.github.ajalt.clikt.core.CliktCommand

class Komac : CliktCommand(invokeWithoutSubcommand = true, printHelpOnEmptyArgs = true) {
    override fun aliases(): Map<String, List<String>> = mapOf(
        "up" to listOf("update"),
        "rm" to listOf("remove")
    )

    override fun run() = Unit
}
