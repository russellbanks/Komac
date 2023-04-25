package commands.interfaces

import com.github.ajalt.mordant.terminal.Terminal
import input.Prompts
import utils.menu.radioMenu

interface RadioMenuPrompt<T> : Prompt<T> {
    val name: String

    val default: T? get() = null

    val items: List<T>

    override suspend fun prompt(terminal: Terminal): T? = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} Enter the ${name.lowercase()}"))
        default?.let { muted("Previous value: $it") }
        return radioMenu<T> {
            items = this@RadioMenuPrompt.items
            default = this@RadioMenuPrompt.default
        }.prompt()
    }

    override suspend fun getError(input: String): String? = null
}
