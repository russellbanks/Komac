package input.menu

import com.github.ajalt.mordant.rendering.Widget

interface Menu<T> {
    val menuWidget: Widget

    val items: List<MenuItem<*>>

    val validIndices: List<Int>

    fun prompt(): T

    enum class Key(val code: Int) {
        Up(65),
        Down(66),
        Enter(13)
    }
}
