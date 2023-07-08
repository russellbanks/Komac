package schemas

import kotlinx.serialization.Serializable

@Serializable
data class GHGraphQLRequestBody(
    val query: String
)
