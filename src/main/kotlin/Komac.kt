
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.sun.jna.Platform
import io.Codepage
import network.Proxy

class Komac : CliktCommand(printHelpOnEmptyArgs = true) {
    private val useSystemProxy: Boolean by option(
        "--use-system-proxy",
        help = "When enabled, Komac will use the system's proxy",
        envvar = "USE_SYSTEM_PROXY"
    ).flag(default = false)

    override fun aliases(): Map<String, List<String>> = mapOf(
        "up" to listOf("update"),
        "rm" to listOf("remove"),
        "delete" to listOf("remove"),
        "cleanup" to listOf("branch", "cleanup")
    )

    override fun run() {
        if (Platform.isWindows()) {
            Codepage.setConsoleUTF8()
            Runtime.getRuntime().addShutdownHook(Thread { Codepage.resetCodepage() })
        }
        if (useSystemProxy) {
            Proxy.useSystemProxy()
        }
    }
}
