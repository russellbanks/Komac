package input.menu

import com.github.ajalt.mordant.terminal.Terminal
import org.jline.terminal.TerminalBuilder
import org.jline.utils.NonBlockingReader

abstract class BaseMenu<R, T>(
    final override val items: List<MenuItem<T>>,
    default: MenuItem<T>? = null,
    val nameConvert: (MenuItem<T>) -> String = MenuItem<T>::toString,
    protected val terminal: Terminal
) : Menu<R> {
    override val validIndices: List<Int> = items.indices.toList()
    protected var selectedIndex: Int = items.indexOf(default).takeIf { it != -1 } ?: 0
    protected val selectedItem get() = items[selectedIndex]

    protected fun handleKeyPress(
        reader: NonBlockingReader,
        shouldBreak: () -> Boolean = { true },
        onEnter: () -> Unit = {}
    ) {
        while (true) {
            when (reader.read()) {
                Menu.Key.Down.code -> move(Menu.Key.Down) { updateAnimation() }
                Menu.Key.Up.code -> move(Menu.Key.Up) { updateAnimation() }
                Menu.Key.Enter.code -> {
                    onEnter()
                    if (shouldBreak()) break
                }
            }
        }
    }

    protected fun Terminal.withJLineTerminal(block: org.jline.terminal.Terminal.() -> R): R {
        val terminal = TerminalBuilder.terminal().apply { enterRawMode() }
        val reader = terminal.reader()
        try {
            return terminal.block()
        } finally {
            reader.close()
            terminal.close()
            cursor.show()
        }
    }

    private fun move(direction: Menu.Key, updateAnimation: (Int) -> Unit) {
        val newIndex = when (direction) {
            Menu.Key.Up -> selectedIndex - 1
            Menu.Key.Down -> selectedIndex + 1
            Menu.Key.Enter -> selectedIndex
        }

        if (newIndex in validIndices) {
            selectedIndex = newIndex
            updateAnimation(selectedIndex)
        }
    }

    protected abstract fun updateAnimation()

    protected abstract fun clearAnimation()
}
