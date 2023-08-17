package io.menu

import com.github.ajalt.mordant.terminal.Terminal

class YesNoMenu(
    default: Boolean = true,
    terminal: Terminal
) : RadioMenu<Boolean>(
    items = listOf(true, false).map { MenuItem.Item(it) },
    default = MenuItem.Item(default),
    nameConvert = { if (it is MenuItem.Item && it.value) "Yes" else "No" },
    terminal = terminal
) {
    override fun prompt(): Boolean = (super.prompt() as Boolean).also { println() }
}

fun Terminal.yesNoMenu(default: Boolean = true): YesNoMenu = YesNoMenu(default, this)
