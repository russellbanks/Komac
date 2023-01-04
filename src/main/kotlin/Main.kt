
import com.github.ajalt.clikt.core.subcommands
import com.github.ajalt.clikt.parameters.options.versionOption
import com.russellbanks.Komac.BuildConfig
import org.koin.core.context.GlobalContext.startKoin
import org.koin.ksp.generated.defaultModule

fun main(args: Array<String>) {
    startKoin {
        defaultModule()
    }

    Komac()
        .subcommands(NewManifest(), QuickUpdate())
        .versionOption(version = BuildConfig.appVersion, names = setOf("-v", "--version"))
        .main(args)
}
