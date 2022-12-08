import org.koin.core.context.GlobalContext.startKoin
import org.koin.ksp.generated.defaultModule

suspend fun main(args: Array<String>) {
    startKoin {
        defaultModule()
    }

    Komac().main(args)
}
