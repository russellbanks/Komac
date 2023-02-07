package commands

import com.github.ajalt.mordant.terminal.Terminal

interface CommandPrompt<T> {
    suspend fun prompt(terminal: Terminal): T

    fun getError(input: String?): String?
}
