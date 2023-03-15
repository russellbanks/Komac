package commands.interfaces

import com.github.ajalt.mordant.terminal.Terminal

interface Prompt<T> {
    suspend fun prompt(terminal: Terminal): T?

    suspend fun getError(input: String): String?
}
