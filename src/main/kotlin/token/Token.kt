package token

import com.github.ajalt.clikt.core.PrintMessage
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
import org.kohsuke.github.HttpException

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
        } ?: throw ProgramResult(ExitCode.CTRLC)
    }

    override suspend fun getError(input: String): String? {
        return if (isTokenValid(input)) null else "Invalid token. Please try again"
    }

    fun isTokenValid(tokenString: String?): Boolean {
        return try {
            GitHub.connectUsingOAuth(tokenString).run {
                checkApiUrlValidity()
                isCredentialValid
            }
        } catch (httpException: HttpException) {
            if (httpException.responseCode == -1) {
                throw PrintMessage(
                    message = TextColors.red("""
                        Komac was unable to connect to GitHub.
                        Please check your internet connection and try again.
                        """.trimIndent()
                    ),
                    statusCode = 1,
                    printError = true
                )
            } else {
                false
            }
        } catch (_: IOException) {
            false
        }
    }
}
