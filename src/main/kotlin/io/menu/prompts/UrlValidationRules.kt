package io.menu.prompts

import io.ktor.http.Url

data class UrlValidationRules(
    val isRequired: Boolean = false,
    val transform: (String) -> Url = { urlString -> Url(urlString.trim()) },
    val checkForRedirect: Boolean = false
)
