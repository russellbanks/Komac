package io.menu.prompts

import Errors
import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import github.GitHubDetection
import io.ExitCode
import io.Prompts
import io.ktor.client.HttpClient
import io.ktor.client.network.sockets.ConnectTimeoutException
import io.ktor.client.request.head
import io.ktor.http.Url
import io.ktor.http.isSuccess
import io.menu.yesNoMenu
import java.net.ConnectException
import kotlinx.coroutines.runBlocking
import network.Http
import utils.getRedirectedUrl
import utils.isRedirect

interface UrlPrompt : Prompt<Url> {
    val description: String

    val previousUrl: Url? get() = null

    val validationRules: UrlValidationRules get() = UrlValidationRules()

    override suspend fun prompt(terminal: Terminal): Url = with(terminal) {
        val textColour = if (validationRules.isRequired) colors.brightGreen else colors.brightYellow
        val requiredText = if (validationRules.isRequired) Prompts.required else Prompts.optional
        println(textColour("$requiredText Enter the $description"))
        return prompt(
            prompt = name,
            default = previousUrl?.also { muted("Previous ${name.lowercase()}: $it") }
        ) { input ->
            runBlocking { getError(input) }
                ?.let { ConversionResult.Invalid(it) }
                ?: ConversionResult.Valid(validationRules.transform(input))
        }?.let {
            if (validationRules.checkForRedirect) {
                println()
                promptIfRedirectedUrl(it)
            } else {
                it
            }
        } ?: throw ProgramResult(ExitCode.CtrlC)
    }

    private suspend fun Terminal.promptIfRedirectedUrl(installerUrl: Url): Url {
        val redirectedUrl = installerUrl.getRedirectedUrl()
        val shouldUseRedirectedUrl = redirectedUrl != installerUrl &&
                !installerUrl.host.equals(other = GitHubDetection.gitHubWebsite, ignoreCase = true)
        val error = getError(redirectedUrl.toString())
        return if (shouldUseRedirectedUrl && error == null) {
            println(colors.brightYellow("The URL is redirected. Would you like to use the destination URL instead?"))
            println(colors.cyan("Discovered URL: $redirectedUrl"))
            if (yesNoMenu(default = true).prompt()) {
                success("URL changed to $redirectedUrl")
                println()
                redirectedUrl
            } else {
                info("Original URL Retained - Proceeding with $installerUrl")
                installerUrl
            }
        } else {
            installerUrl
        }
    }

    override suspend fun getError(input: String): String? {
        val url = validationRules.transform(input)
        return when {
            url == Url("") && !validationRules.isRequired -> null
            url == Url("") -> Errors.blankInput(name)
            url.toString().length > maxLength -> Errors.invalidLength(max = maxLength)
            !url.toString().matches(regex) -> Errors.invalidRegex(regex)
            else -> Http.client.checkUrlResponse(url)
        }
    }

    private suspend fun HttpClient.checkUrlResponse(url: Url): String? = config { followRedirects = false }.use {
        try {
            val installerUrlResponse = it.head(url)
            if (!installerUrlResponse.status.isSuccess() && !installerUrlResponse.status.isRedirect) {
                Errors.unsuccessfulUrlResponse(installerUrlResponse)
            } else {
                null
            }
        } catch (_: ConnectTimeoutException) {
            Errors.connectionTimeout
        } catch (_: ConnectException) {
            Errors.connectionFailure
        }
    }

    companion object {
        private const val maxLength = 2048
        private const val pattern = "^([Hh][Tt][Tt][Pp][Ss]?)://.+$"
        val regex = Regex(pattern)
    }
}
