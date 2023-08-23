package io

enum class ManifestResultOption {
    PullRequest,
    WriteToFiles,
    Quit;

    override fun toString() = name
        .replace("(?<=.)(?=\\p{Lu})".toRegex(), " ")
        .lowercase()
        .replaceFirstChar(Char::titlecase)
}
