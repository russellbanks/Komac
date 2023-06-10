package io.menu

import com.github.ajalt.mordant.terminal.Terminal

class YesNoMenu(
    default: Boolean = true,
    terminal: Terminal
) : RadioMenu<Boolean>(
    items = listOf(MenuItem.Item(true), MenuItem.Item(false)),
    default = MenuItem.Item(default),
    nameConvert = { if (it is MenuItem.Item && it.value) "Yes" else "No" },
    terminal = terminal
) {
    override fun prompt(): Boolean = super.prompt() as Boolean
}

fun Terminal.yesNoMenu(default: Boolean = true): YesNoMenu = YesNoMenu(default, this)
