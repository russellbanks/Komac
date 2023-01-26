package commands.token

import com.github.ajalt.clikt.core.CliktCommand
import org.koin.core.component.KoinComponent

class Token : CliktCommand(printHelpOnEmptyArgs = true), KoinComponent {
    override fun aliases(): Map<String, List<String>> = mapOf(
        "up" to listOf("update"),
        "rm" to listOf("remove"),
        "delete" to listOf("remove")
    )

    override fun run() = Unit
}
