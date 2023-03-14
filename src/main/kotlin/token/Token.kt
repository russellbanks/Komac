package token

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import commands.CommandPrompt
import input.ExitCode
import org.kohsuke.github.GitHub
import java.io.IOException

object Token : CommandPrompt<String> {
    override fun prompt(terminal: Terminal): String {
        return terminal.prompt(
            prompt = terminal.colors.brightGreen("Please enter your GitHub personal access token"),
            hideInput = true
        ) { input ->
            getError(input)
                ?.let { ConversionResult.Invalid(it) }
                ?: ConversionResult.Valid(input.trim())
        } ?: throw ProgramResult(ExitCode.CtrlC.code)
    }

    override fun getError(input: String?): String? {
        return if (input == null) {
            null
        } else {
            try {
                if (isTokenValid(input)) null else "Invalid token. Please try again"
            } catch (_: IOException) {
                "Invalid token. Please try again"
            }
        }
    }

    fun isTokenValid(tokenString: String?): Boolean {
        return try {
            GitHub.connectUsingOAuth(tokenString).isCredentialValid
        } catch (_: IOException) {
            false
        }
    }
}
