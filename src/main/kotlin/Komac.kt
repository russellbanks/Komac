
import com.github.ajalt.clikt.core.CliktCommand
import kotlinx.coroutines.runBlocking
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.context.startKoin
import org.koin.ksp.generated.defaultModule
import token.TokenStore

class Komac : CliktCommand(invokeWithoutSubcommand = true, printHelpOnEmptyArgs = true), KoinComponent {
    override fun aliases(): Map<String, List<String>> = mapOf(
        "up" to listOf("update"),
        "rm" to listOf("remove")
    )

    override fun run(): Unit = runBlocking {
        startKoin {
            defaultModule()
        }
        get<TokenStore>()
    }
}
