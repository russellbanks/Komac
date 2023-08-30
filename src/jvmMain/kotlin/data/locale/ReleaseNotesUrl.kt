package data.locale

import io.menu.prompts.UrlPrompt

object ReleaseNotesUrl : UrlPrompt {
    override val name: String = "Release notes url"

    override val description: String = "package release notes url"
}
