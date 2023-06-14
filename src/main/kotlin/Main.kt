
import com.github.ajalt.clikt.core.subcommands
import com.github.ajalt.clikt.parameters.options.versionOption
import com.russellbanks.Komac.BuildConfig
import commands.NewManifest
import commands.QuickUpdate
import commands.RemoveVersion
import commands.branch.Branch
import commands.branch.Cleanup
import commands.token.Remove
import commands.token.Token
import commands.token.Update

fun main(args: Array<String>) {
    Komac()
        .subcommands(
            NewManifest(),
            QuickUpdate(),
            RemoveVersion(),
            Token().subcommands(Update(), Remove()),
            Branch().subcommands(Cleanup())
        )
        .versionOption(version = BuildConfig.appVersion, names = setOf("-v", "--version"))
        .main(args)
}
