package utils.menu

import com.github.ajalt.mordant.animation.animation
import com.github.ajalt.mordant.rendering.Widget
import com.github.ajalt.mordant.table.verticalLayout
import com.github.ajalt.mordant.terminal.Terminal

class CheckMenu<T>(
    items: List<MenuItem<T>>,
    defaultChecked: List<T> = emptyList(),
    nameConvert: (MenuItem<T>) -> String = MenuItem<T>::toString,
    terminal: Terminal
) : BaseMenu<List<T>?, T>(items, nameConvert = nameConvert, terminal = terminal) {
    override val validIndices: List<Int> = items.indices + items.size
    private val selectedIndices = items.indices
        .filter { index -> items[index] in defaultChecked.map { MenuItem.Item(it) } }
        .toMutableList()
    private val animation = terminal.animation<Int> { menuWidget }

    override val menuWidget: Widget
        get() = verticalLayout {
            items.forEachIndexed { index, item ->
                val isHighlighted = index == selectedIndex
                val isSelected = index in selectedIndices
                val highlightedColour = if (isHighlighted) terminal.colors.brightMagenta else terminal.colors.plain
                cell(
                    "${highlightedColour("[")}${if (isSelected) "x" else " "}${highlightedColour("]")} ${
                        nameConvert(item)
                    }"
                )
            }
            val confirmSelectedColour = if (selectedIndex == items.size) {
                terminal.colors.brightMagenta
            } else {
                terminal.colors.plain
            }
            cell(confirmSelectedColour("[Confirm]"))
        }

    private val confirmPressed get() = selectedIndex == items.size

    override fun prompt(): List<T>? = terminal.withJLineTerminal {
        terminal.cursor.hide(showOnExit = true)
        updateAnimation()
        handleKeyPress(reader(), shouldBreak = { confirmPressed }) {
            if (confirmPressed) {
                clearAnimation()
                terminal.println(menuWidget)
            } else {
                toggleSelected()
            }
        }

        selectedIndices.mapNotNull { index ->
            when (val item = items[index]) {
                is MenuItem.Item -> item.value
                is MenuItem.Optional -> null
            }
        }.ifEmpty { null }
    }

    private fun toggleSelected() {
        if (selectedIndex in selectedIndices) {
            selectedIndices.remove(selectedIndex)
        } else {
            selectedIndices.add(selectedIndex)
        }
        updateAnimation()
    }

    override fun updateAnimation() = animation.update(selectedIndex)

    override fun clearAnimation() = animation.clear()
}

class CheckMenuBuilder<T> {
    lateinit var items: List<T>
    var defaultChecked: List<T> = emptyList()
    var nameConvert: (MenuItem<T>) -> String = { item ->
        when (item) {
            is MenuItem.Item -> item.value.toString()
            MenuItem.Optional -> item.toString()
        }
    }
}

fun <T> Terminal.checkMenu(block: CheckMenuBuilder<T>.() -> Unit): CheckMenu<T> {
    val builder = CheckMenuBuilder<T>().apply(block)
    val items = builder.items.map { MenuItem.Item(it) }
    return CheckMenu(items, builder.defaultChecked, { item -> builder.nameConvert(item) }, this)
}
