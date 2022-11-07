package schemas

import kotlinx.serialization.Serializable

@Serializable(with = ManifestSerializer::class)
sealed class Schema
