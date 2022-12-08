import com.github.ajalt.clikt.parameters.options.versionOption
import com.russellbanks.Komac.BuildConfig
import org.koin.core.context.GlobalContext.startKoin
import org.koin.ksp.generated.defaultModule

suspend fun main(args: Array<String>) {
    startKoin {
        defaultModule()
    }

    Komac().versionOption(version = BuildConfig.appVersion, names = setOf("-v")).main(args)
}
