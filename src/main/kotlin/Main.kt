
import com.github.ajalt.clikt.core.subcommands
import com.github.ajalt.clikt.parameters.options.versionOption
import com.russellbanks.Komac.BuildConfig
import commands.ChangeToken
import commands.NewManifest
import commands.QuickUpdate
import commands.RemoveVersion
import org.koin.core.context.GlobalContext.startKoin
import org.koin.ksp.generated.defaultModule

fun main(args: Array<String>) {
    startKoin {
        defaultModule()
    }

    Komac(args)
        .subcommands(NewManifest(), QuickUpdate(), RemoveVersion(), ChangeToken())
        .versionOption(version = BuildConfig.appVersion, names = setOf("-v", "--version"))
        .main(args)
}
