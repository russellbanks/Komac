
import com.github.ajalt.clikt.core.subcommands
import com.github.ajalt.clikt.parameters.options.versionOption
import com.russellbanks.Komac.BuildConfig
import commands.ChangeToken
import commands.NewManifest
import commands.QuickUpdate
import commands.RemoveVersion

fun main(args: Array<String>) {
    Komac()
        .subcommands(NewManifest(), QuickUpdate(), RemoveVersion(), ChangeToken())
        .versionOption(version = BuildConfig.appVersion, names = setOf("-v", "--version"))
        .main(args)
}
