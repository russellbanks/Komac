package token

import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.terminal.Terminal
import com.microsoft.alm.secret.Credential
import com.microsoft.alm.storage.windows.CredManagerBackedCredentialStore
import org.kohsuke.github.GitHubBuilder
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import java.io.IOException

@Single
class TokenStore : KoinComponent {
    lateinit var token: String
    fun getToken(terminal: Terminal) {
        if (System.getProperty(osName).startsWith(windows)) {
            val credentialStore = CredManagerBackedCredentialStore()
            if (credentialStore[credentialKey] == null) {
                terminal.promptForToken().also {
                    credentialStore.add(credentialKey, Credential(credentialUsername, it))
                    token = it
                }
            } else {
                token = credentialStore[credentialKey].Password
            }
        }
    }

    private fun Terminal.promptForToken(): String {
        var token: String?
        do {
            println(brightGreen("Please enter your GitHub personal access token:"))
            token = prompt(brightWhite("Token"))
            val isValid = try {
                GitHubBuilder().withOAuthToken(get<TokenStore>().token).build()
                true
            } catch (_: IOException) {
                println(brightGreen("Invalid token. Please try again."))
                false
            }
        } while (token == null || !isValid)
        return token
    }

    companion object {
        private const val osName = "os.name"
        private const val windows = "Windows"
        private const val credentialUsername = "github-access-token"
        private const val credentialKey = "komac/$credentialUsername"
    }
}
