package input

enum class ManifestResultOption {
    PullRequest,
    WriteToFiles,
    Quit;

    override fun toString() = name.replace(Regex("([A-Z])"), " $1").trim()
}
