package utils

import com.github.ajalt.mordant.animation.animation
import com.github.ajalt.mordant.rendering.Widget
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import input.ExitCode
import org.jline.terminal.TerminalBuilder
import kotlin.system.exitProcess
class MenuCreator<T>(private val items: List<T>, default: T? = null, private val terminal: Terminal) {
    private var selectedIndex = items.indexOf(default).takeIf { it != -1 } ?: 0
    private val selectedItem
        get() = items[selectedIndex]

    private val menuWidget: Widget
        get() {
            return with(terminal) {
                verticalLayout {
                    items.forEachIndexed { index, item ->
                        val isSelected = index == selectedIndex
                        val selectedColour = if (isSelected) colors.magenta else colors.plain
                        cell(selectedColour("[${if (isSelected) "x" else " "}] $item"))
                    }
                }
            }
        }

    fun prompt(): T = with(terminal) {
        val animation = animation<T> { menuWidget }
        cursor.hide(showOnExit = true)
        animation.update(selectedItem)
        val chosenItem: T
        val terminal = TerminalBuilder.terminal().apply {
            enterRawMode()
            handle(org.jline.terminal.Terminal.Signal.INT) { exitProcess(ExitCode.CtrlC.code) }
        }
        val reader = terminal.reader()
        while (true) {
            when (reader.read()) {
                65 -> {
                    if (selectedIndex.dec() >= 0) {
                        selectedIndex--
                        animation.update(selectedItem)
                    }
                }
                66 -> {
                    if (selectedIndex.inc() <= items.lastIndex) {
                        selectedIndex++
                        animation.update(selectedItem)
                    }
                }
                13 -> {
                    animation.clear()
                    println(menuWidget)
                    chosenItem = selectedItem
                    break
                }
            }
        }
        try {
            return chosenItem
        } finally {
            reader.close()
            terminal.close()
            cursor.show()
        }
    }
}

fun <T> Terminal.menu(items: List<T>, default: T? = null): MenuCreator<T> = MenuCreator(items, default, this)

