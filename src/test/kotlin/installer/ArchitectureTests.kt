package installer

import InstallerSchemaData
import Validation
import data.Architecture.isArchitectureValid
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe

class ArchitectureTests : FunSpec({
    val installerSchema = InstallerSchemaData.installerSchema

    context("Architecture Tests") {
        val architectureSchema = installerSchema.definitions.architecture
        withData(architectureSchema.enum) {
            isArchitectureValid(it, architectureSchema).first shouldBe Validation.Success
        }

        withData(
            listOf("64", "86", "x32", "64bit", "32bit", "arm32", "arm32bit", "arm64bit", "x64bit", null)
        ) {
            isArchitectureValid(it, architectureSchema).first shouldNotBe Validation.Success
        }
    }
})
