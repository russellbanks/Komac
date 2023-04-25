package utils

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.animation.Animation
import com.github.ajalt.mordant.animation.animation
import com.github.ajalt.mordant.rendering.Widget
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal
import input.ExitCode
import org.jline.terminal.TerminalBuilder

open class Menu<T>(
    private val items: List<MenuItem<T>>,
    default: MenuItem<T>? = null,
    private val nameConvert: (MenuItem<T>) -> String = { it.toString() },
    private val terminal: Terminal
) {
    private var selectedIndex = items.indexOf(default).takeIf { it != -1 } ?: 0
    private val selectedItem get() = items[selectedIndex]

    private val menuWidget: Widget
        get() = verticalLayout {
            items.forEachIndexed { index, item ->
                val isSelected = index == selectedIndex
                val selectedColour = if (isSelected) terminal.colors.brightMagenta else terminal.colors.plain
                cell("${selectedColour("[${if (isSelected) "x" else " "}]")} ${nameConvert(item)}")
            }
        }

    open fun prompt(): T? = with(terminal) {
        val animation = animation<MenuItem<T>> { menuWidget }
        cursor.hide(showOnExit = true)
        animation.update(selectedItem)
        val terminal = TerminalBuilder.terminal().apply { enterRawMode() }
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
            return when (val item = selectedItem) {
                is MenuItem.Item -> item.value
                is MenuItem.Optional -> null
            }
        } finally {
            reader.close()
            terminal.close()
            cursor.show()
        }
    }

    private fun move(animation: Animation<MenuItem<T>>, direction: Key) {
        val newIndex = when (direction) {
            Key.Up -> selectedIndex - 1
            Key.Down -> selectedIndex + 1
            Key.Enter -> selectedIndex
        }

        if (newIndex in items.indices) {
            selectedIndex = newIndex
            animation.update(selectedItem)
        }
    }

    private enum class Key(val code: Int) {
        Up(65),
        Down(66),
        Enter(13)
    }
}

sealed class MenuItem<out T> {
    data class Item<T>(val value: T) : MenuItem<T>()
    data object Optional : MenuItem<Nothing>()
}

class MenuBuilder<T> {
    lateinit var items: List<T>
    var default: T? = null
    var optionalItemName: String? = null
    var nameConvert: (MenuItem<T>, String?) -> String = { item, optionalName ->
        when (item) {
            is MenuItem.Item -> item.value.toString()
            MenuItem.Optional -> optionalName ?: item.toString()
        }
    }

    internal fun buildMenuItems(): List<MenuItem<T>> {
        val menuItems = items.map { MenuItem.Item(it) }
        return if (optionalItemName != null) {
            menuItems + MenuItem.Optional
        } else {
            menuItems
        }
    }
}

class YesNoMenu(
    default: Boolean = true,
    terminal: Terminal
) : Menu<Boolean>(
    items = listOf(MenuItem.Item(true), MenuItem.Item(false)),
    default = MenuItem.Item(default),
    nameConvert = { if (it is MenuItem.Item && it.value) "Yes" else "No" },
    terminal = terminal
) {
    override fun prompt(): Boolean = super.prompt()!!
}

fun <T> Terminal.menu(block: MenuBuilder<T>.() -> Unit): Menu<T> {
    val builder = MenuBuilder<T>().apply(block)
    val items = builder.buildMenuItems()
    val defaultItem = builder.default?.let { MenuItem.Item(it) }
    return Menu(items, defaultItem, { item -> builder.nameConvert(item, builder.optionalItemName) }, this)
}


fun Terminal.yesNoMenu(default: Boolean = true): YesNoMenu = YesNoMenu(default, this)
