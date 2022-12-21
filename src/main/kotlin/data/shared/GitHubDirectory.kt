package data.shared

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

class GitHubDirectory : ArrayList<GitHubDirectory.GitHubDirectoryItem>() {
    @Serializable
    data class GitHubDirectoryItem(
        @SerialName("name") val name: String,
        @SerialName("path") val path: String,
        @SerialName("sha") val sha: String,
        @SerialName("size") val size: Int,
        @SerialName("url") val url: String,
        @SerialName("html_url") val htmlUrl: String,
        @SerialName("git_url") val gitUrl: String,
        @SerialName("download_url") val downloadUrl: String?,
        @SerialName("type") val type: String,
        @SerialName("_links") val links: Links
    ) {
        @Serializable
        data class Links(
            @SerialName("self") val self: String,
            @SerialName("git") val git: String,
            @SerialName("html") val html: String
        )
    }
}
