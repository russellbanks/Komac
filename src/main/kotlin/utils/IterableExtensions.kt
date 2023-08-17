package utils

/**
 * Transforms each element of the iterable using the provided [selector] function, collects the transformed elements
 * into a set, and then evaluates whether there is a single distinct element in the set. If there's only one distinct
 * element, returns `null`; otherwise, returns the specified [default] value.
 *
 * @param default the default value to return if there is more than one distinct element.
 * @param selector the function to transform elements.
 * @return `null` if there is a single distinct element; otherwise, the [default] value.
 */
inline fun <T, R> Iterable<T>.filterSingleDistinctOrElse(default: R, selector: (T) -> R): R? {
    return if (mapTo(HashSet(), selector).size == 1) null else default
}

/**
 * Maps each element of the iterable using the provided [selector] function, collects the transformed elements into a
 * set, and then returns a single distinct element from the set, or null if the set is empty or contains more than one
 * element.
 *
 * @param selector the function to transform elements.
 * @return a single distinct element from the transformed set, or null if the set is empty or contains more than one
 * element.
 */
inline fun <T, R> Iterable<T>.mapDistinctSingleOrNull(selector: (T) -> R): R? {
    return mapTo(HashSet(), selector).singleOrNull()
}
