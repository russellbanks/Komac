package utils

/**
 * Returns the default value if the values returned by the [selector] function on each element
 * in this iterable are all the same. Otherwise, returns `null`.
 *
 * @param default the value to return if the elements are not distinct
 * @param selector a function that maps elements to a value to be compared for distinctness
 * @return the default value if the elements are not distinct, `null` otherwise
 */
inline fun <T, R> Iterable<T>.takeIfNotDistinct(default: R, selector: (T) -> R): R? {
    return if (any()) {
        if (distinctBy(selector).size == 1) null else default
    } else {
        null
    }
}

/**
 * Returns a distinct value of type [R] obtained by applying the given [selector] function to each element of the
 * iterable, or null if there are multiple distinct values or if there are no values after applying the [selector].
 *
 * @param selector a function that maps an element of the iterable to a value of type [R] or returns null if the
 * element should be skipped.
 * @return a distinct value of type [R] or null if there are multiple distinct values or if there are no values
 * after applying the [selector].
 */
inline fun <T, R> Iterable<T>.getDistinctOrNull(selector: (T) -> R?): R? {
    return mapNotNull(selector).toSet().singleOrNull()
}
