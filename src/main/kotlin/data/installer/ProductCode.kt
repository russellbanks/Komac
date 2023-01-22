package data.installer

import Errors
import ExitCode
import com.github.ajalt.mordant.terminal.ConversionResult
import com.github.ajalt.mordant.terminal.Terminal
import data.InstallerManifestData
import data.SharedManifestData
import input.Prompts
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import java.util.UUID
import kotlin.system.exitProcess

object ProductCode : KoinComponent {
    private val sharedManifestData: SharedManifestData by inject()
    private val installerManifestData: InstallerManifestData by inject()
    private val productCodeSchema = get<SchemasImpl>().installerSchema.definitions.productCode

    fun Terminal.productCodePrompt() {
        sharedManifestData.msi?.productCode?.let {
            installerManifestData.productCode = it
            return
        }
        println(colors.brightYellow(productCodeInfo))
        info(example)
        installerManifestData.productCode = prompt(
            prompt = const,
            convert = { input ->
                isProductCodeValid(input)
                    ?.let { ConversionResult.Invalid(it) }
                    ?: ConversionResult.Valid(input.trim().uppercase().ensureProductCodeBrackets())
            }
        ) ?: exitProcess(ExitCode.CtrlC.code)
        println()
    }

    private fun isProductCodeValid(productCode: String): String? {
        return when {
            productCode.isNotBlank() && productCode.length > productCodeSchema.maxLength -> {
                Errors.invalidLength(min = productCodeSchema.minLength, max = productCodeSchema.maxLength)
            }
            !productCode.matches(Regex(pattern)) -> Errors.invalidRegex(regex = Regex(pattern))
            else -> null
        }
    }

    private fun String.ensureProductCodeBrackets(): String {
        return when {
            startsWith("{") && endsWith("}") -> this
            startsWith("{") -> "$this}"
            endsWith("}") -> "{$this"
            else -> "{$this}"
        }
    }

    private const val const = "Product Code"
    private const val pattern =
        "^\\{?[0-9a-fA-F]{8}\\b-[0-9a-fA-F]{4}\\b-[0-9a-fA-F]{4}\\b-[0-9a-fA-F]{4}\\b-[0-9a-fA-F]{12}}?\$"
    private const val productCodeInfo = "${Prompts.optional} Enter the application product code."
    private val example = "Looks like: {${UUID.randomUUID().toString().uppercase()}}"
}
