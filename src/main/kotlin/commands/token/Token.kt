package commands.token

import com.github.ajalt.clikt.core.CliktCommand

class Token : CliktCommand(printHelpOnEmptyArgs = true) {
    override fun aliases(): Map<String, List<String>> = mapOf(
        "up" to listOf("update"),
        "rm" to listOf("remove"),
        "delete" to listOf("remove")
    )

    override fun run() = Unit
}
