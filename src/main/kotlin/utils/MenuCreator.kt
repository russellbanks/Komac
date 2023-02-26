package utils

import com.github.ajalt.mordant.animation.Animation
import com.github.ajalt.mordant.animation.animation
import com.github.ajalt.mordant.rendering.Widget
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import input.ExitCode
import org.jline.terminal.TerminalBuilder
import kotlin.system.exitProcess

@Suppress("UNCHECKED_CAST")
class MenuCreator<T : Any>(
    items: List<T>,
    default: Any? = null,
    private val optionalItemName: String? = null,
    private val nameConvert: (String) -> String = { it },
    private val terminal: Terminal
) {
    private val listItems: List<Any> = optionalItemName?.let { items + it } ?: items
    private var selectedIndex = listItems.indexOf(default ?: optionalItemName).takeIf { it != -1 } ?: 0
    private val selectedItem
        get() = listItems[selectedIndex]

    private val menuWidget: Widget
        get() {
            return verticalLayout {
                listItems.forEachIndexed { index, item ->
                    val isSelected = index == selectedIndex
                    val selectedColour = if (isSelected) terminal.colors.magenta else terminal.colors.plain
                    cell(selectedColour("[${if (isSelected) "x" else " "}] ${nameConvert(item.toString())}"))
                }
            }
        }

    fun prompt(): T? = with(terminal) {
        val animation = animation<Any> { menuWidget }
        cursor.hide(showOnExit = true)
        animation.update(selectedItem)
        val terminal = TerminalBuilder.terminal().apply {
            enterRawMode()
            handle(org.jline.terminal.Terminal.Signal.INT) { exitProcess(ExitCode.CtrlC.code) }
        }
        val reader = terminal.reader()
        while (true) {
            when (reader.read()) {
                Key.Down.code -> move(animation, Key.Down)
                Key.Up.code -> move(animation, Key.Up)
                Key.Enter.code -> {
                    animation.clear()
                    println(menuWidget)
                    break
                }
            }
        }
        try {
            return if (selectedItem == optionalItemName) null else selectedItem as T
        } finally {
            reader.close()
            terminal.close()
            cursor.show()
        }
    }

    private fun move(animation: Animation<Any>, direction: Key) {
        if (direction == Key.Up) {
            if (selectedIndex.dec() >= 0) {
                selectedIndex--
                animation.update(selectedItem)
            }
        } else if (direction == Key.Down) {
            if (selectedIndex.inc() <= listItems.lastIndex) {
                selectedIndex++
                animation.update(selectedItem)
            }
        }
    }

    private enum class Key(val code: Int) {
        Up(65),
        Down(66),
        Enter(13)
    }
}

fun <T : Any> Terminal.menu(
    items: List<T>,
    default: T? = null,
    optionalItemName: String? = null,
    nameConvert: (String) -> String = { it }
): MenuCreator<T> {
    return MenuCreator(items, default, optionalItemName, nameConvert, this)
}

fun Terminal.yesNoMenu(default: Boolean? = null): Boolean {
    return menu(
        items = YesNo.values().toList(),
        default = if (default == true) YesNo.Yes else YesNo.No
    ).prompt()!!.toBoolean()
}

enum class YesNo {
    Yes,
    No;

    fun toBoolean() = this == Yes
}
