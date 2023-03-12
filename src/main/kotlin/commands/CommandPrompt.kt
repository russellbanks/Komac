package commands

import com.github.ajalt.mordant.terminal.Terminal

interface CommandPrompt<T> {
    fun prompt(terminal: Terminal): T?

    fun getError(input: String?): String?
}
