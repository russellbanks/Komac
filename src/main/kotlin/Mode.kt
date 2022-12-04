enum class Mode {
    NewManifest { override fun toString() = "New Manifest or Package Version" },
    QuickUpdate { override fun toString() = "Quick Update Package Version" },
    UpdateMetadata { override fun toString() = "Update Package Metadata" },
    NewLocale { override fun toString() = "New Locale" },
    RemoveManifest { override fun toString() = "Remove a manifest" },
    Exit { override fun toString() = "Press Q to quit" }
}
