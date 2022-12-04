enum class Mode(val key: Char) {
    NewManifest('1') { override fun toString() = "New Manifest or Package Version" },
    QuickUpdate('2') { override fun toString() = "Quick Update Package Version" },
    UpdateMetadata('3') { override fun toString() = "Update Package Metadata" },
    NewLocale('4') { override fun toString() = "New Locale" },
    RemoveManifest('5') { override fun toString() = "Remove a manifest" },
    Exit('Q') { override fun toString() = "Press Q to quit" }
}
