package commands.interfaces

import com.github.ajalt.mordant.terminal.Terminal
import utils.menu

interface MenuPrompt<T> : Prompt<T> {
    val name: String

    val default: T? get() = null

    val items: List<T>

    override suspend fun prompt(terminal: Terminal): T? = with(terminal) {
        println(colors.brightYellow("Enter the ${name.lowercase()}"))
        default?.let { muted("Previous value: $it") }
        return menu<T> {
            items = this@MenuPrompt.items
            default = this@MenuPrompt.default
        }.prompt()
    }

    override suspend fun getError(input: String): String? = null
}
