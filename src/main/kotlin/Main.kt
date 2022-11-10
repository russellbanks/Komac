import org.koin.core.context.GlobalContext.startKoin
import org.koin.ksp.generated.defaultModule

suspend fun main() {

    startKoin {
        defaultModule()
    }

    Application().main()

}
