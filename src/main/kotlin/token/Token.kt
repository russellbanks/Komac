package token

import com.github.ajalt.clikt.core.ProgramResult
import com.github.ajalt.mordant.rendering.TextColors
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import io.ExitCode
import io.menu.prompts.TextPrompt
import io.menu.prompts.ValidationRules
import java.io.IOException
import kotlinx.coroutines.runBlocking
import org.kohsuke.github.GitHub

object Token : TextPrompt {
    override val name: String = "Token"

    override val validationRules: ValidationRules = ValidationRules()

    override suspend fun prompt(terminal: Terminal): String {
        return terminal.prompt(
            prompt = TextColors.brightGreen("Please enter your GitHub personal access token"),
            hideInput = true
        ) { input ->
            runBlocking {
                getError(input.trim())
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim())
            }
        } ?: throw ProgramResult(ExitCode.CtrlC)
    }

    override suspend fun getError(input: String): String? {
        return try {
            if (isTokenValid(input)) null else "Invalid token. Please try again"
        } catch (_: IOException) {
            "Invalid token. Please try again"
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
