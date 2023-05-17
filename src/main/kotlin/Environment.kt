object Environment {
    val isCI = System.getenv("CI")?.toBooleanStrictOrNull() == true
}
