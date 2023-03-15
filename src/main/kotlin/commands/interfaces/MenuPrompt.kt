package commands.interfaces

import com.github.ajalt.mordant.terminal.Terminal
import utils.menu

interface MenuPrompt<T> : Prompt<T> {
    val name: String

    val default: T? get() = null

    val items: List<T>

    override suspend fun prompt(terminal: Terminal): T? = with(terminal) {
        println(colors.brightYellow("Enter the ${name.lowercase()}"))
        if (default != null) println(colors.muted("Previous value: $default"))
        return menu(
            items = items,
            default = default
        ).prompt()
    }

    override suspend fun getError(input: String): String? = null
}
