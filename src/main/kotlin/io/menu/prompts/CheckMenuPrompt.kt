package io.menu.prompts

import com.github.ajalt.mordant.terminal.Terminal
import io.Prompts
import io.menu.checkMenu

interface CheckMenuPrompt<T> : Prompt<List<T>> {
    val defaultChecked: List<T> get() = emptyList()

    val items: List<T>

    override suspend fun prompt(terminal: Terminal): List<T>? = with(terminal) {
        println(colors.brightYellow("${Prompts.optional} Select the ${name.lowercase()}"))
        return checkMenu {
            items = this@CheckMenuPrompt.items
            defaultChecked = this@CheckMenuPrompt.defaultChecked.orEmpty()
        }.prompt()
    }

    override suspend fun getError(input: String): String? = null
}
