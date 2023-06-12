package io.menu

sealed class MenuItem<out T> {
    data class Item<T>(val value: T) : MenuItem<T>()
    data object Optional : MenuItem<Nothing>()
}
