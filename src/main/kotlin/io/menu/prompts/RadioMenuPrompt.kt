package io.menu.prompts

import com.github.ajalt.mordant.terminal.Terminal
import io.Prompts
import io.menu.radioMenu

interface RadioMenuPrompt<T> : Prompt<T> {
    val default: T? get() = null

    val items: List<T>

    override suspend fun prompt(terminal: Terminal): T? = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} Enter the ${name.lowercase()}"))
        return radioMenu<T> {
            items = this@RadioMenuPrompt.items
            default = this@RadioMenuPrompt.default
        }.prompt()
    }

    override suspend fun getError(input: String): String? = null
}
