package schemas

import com.github.ajalt.mordant.terminal.Terminal
import org.koin.core.annotation.Single

@Single
class TerminalInstance {
    val terminal = Terminal()
}