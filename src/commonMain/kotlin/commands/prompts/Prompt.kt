package commands.prompts

import com.github.ajalt.mordant.terminal.Terminal

interface Prompt<T> {
    val name: String

    suspend fun prompt(terminal: Terminal): T?

    suspend fun getError(input: String): String?
}
