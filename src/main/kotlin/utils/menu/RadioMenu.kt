package utils.menu

import com.github.ajalt.mordant.animation.animation
import com.github.ajalt.mordant.rendering.Widget
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal

open class RadioMenu<T>(
    items: List<MenuItem<T>>,
    default: MenuItem<T>? = null,
    nameConvert: (MenuItem<T>) -> String = MenuItem<T>::toString,
    terminal: Terminal
) : BaseMenu<T?, T>(items, default, nameConvert, terminal) {
    private val animation = terminal.animation<MenuItem<T>> { menuWidget }

    override val menuWidget: Widget
        get() = verticalLayout {
            items.forEachIndexed { index, item ->
                val isSelected = index == selectedIndex
                val selectedColour = if (isSelected) terminal.colors.brightMagenta else terminal.colors.plain
                cell("${selectedColour("[${if (isSelected) "x" else " "}]")} ${nameConvert(item)}")
            }
        }

    override fun prompt(): T? = terminal.withJLineTerminal {
        terminal.cursor.hide(showOnExit = true)
        updateAnimation()
        handleKeyPress(reader()) {
            clearAnimation()
            terminal.println(menuWidget)
        }

        when (val item = selectedItem) {
            is MenuItem.Item -> item.value
            is MenuItem.Optional -> null
        }
    }

    override fun updateAnimation() = animation.update(selectedItem)

    override fun clearAnimation() = animation.clear()
}

class RadioMenuBuilder<T> {
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


fun <T> Terminal.radioMenu(block: RadioMenuBuilder<T>.() -> Unit): RadioMenu<T> {
    val builder = RadioMenuBuilder<T>().apply(block)
    val items = builder.buildMenuItems()
    val defaultItem = builder.default?.let { MenuItem.Item(it) }
    return RadioMenu(items, defaultItem, { item -> builder.nameConvert(item, builder.optionalItemName) }, this)
}
