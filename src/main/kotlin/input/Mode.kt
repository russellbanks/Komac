package input

enum class Mode(val key: Char) {
    NewManifest('1') { override fun toString() = "New Manifest or Package Version" },
    QuickUpdate('2') { override fun toString() = "Quick Update Package Version" },
    RemoveVersion('3') { override fun toString() = "Remove a version" },
    Token('4') { override fun toString() = "Change Token" },
    Exit('Q') { override fun toString() = "Press Q to quit" }
}
