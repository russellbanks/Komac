package schemas

import org.koin.core.component.KoinComponent
import org.koin.core.component.get

object Enum : KoinComponent {
    fun upgradeBehaviour(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): List<String> {
        return installerSchema.definitions.upgradeBehavior.enum
    }

    fun architecture(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): List<String> {
        return installerSchema.definitions.architecture.enum
    }

    fun installerType(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): List<String> {
        return installerSchema.definitions.installerType.enum
    }

    fun installerScope(installerSchema: InstallerSchema = get<InstallerSchemaImpl>().installerSchema): List<String> {
        return installerSchema.definitions.scope.enum
    }
}
