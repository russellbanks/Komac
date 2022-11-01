import com.github.ajalt.mordant.rendering.TextColors.blue
import com.github.ajalt.mordant.rendering.TextColors.brightGreen
import com.github.ajalt.mordant.rendering.TextColors.brightWhite
import com.github.ajalt.mordant.rendering.TextColors.red
import com.github.ajalt.mordant.rendering.TextColors.yellow
import com.github.ajalt.mordant.terminal.Terminal
import kotlin.system.exitProcess

fun main() {
    fun optionBox(char: Char): String {
        return "${blue("[")}${brightWhite(char.toString())}${blue("]")}"
    }

    with(Terminal()) {
        println(yellow("Select mode:"))
        println("   ${optionBox('1')} ${blue("New Manifest or Package Version")}")
        println("   ${optionBox('2')} ${blue("Quick Update Package Version")}")
        println("   ${optionBox('3')} ${blue("Update Package Metadata")}")
        println("   ${optionBox('4')} ${blue("New Locale")}")
        println("   ${optionBox('5')} ${blue("Remove a manifest")}")
        println("   ${optionBox('Q')} ${red("Any key to quit")}")
        val selection = prompt(brightWhite("Selection"))
        println("\n")
        when(selection) {
            "1" -> NewManifest(this).run()
            "2" -> TODO()
            "3" -> TODO()
            "4" -> TODO()
            "5" -> TODO()
            else -> exitProcess(0)
        }
    }
}

class NewManifest(private val terminal: Terminal) {
    fun run() {
        packageIdentifierPrompt()
        println()
        packageVersionPrompt()
    }

    private fun packageIdentifierPrompt() {
        with(terminal) {
            var packageIdentifierSuccessful = false
            println(brightGreen("[Required] Enter the Package Identifier, in the following format <Publisher shortname.Application shortname>. For example: Microsoft.Excel"))
            var packageIdentifier = prompt(brightWhite("Package Identifier"))?.trim()
            while(!packageIdentifierSuccessful) {
                if ((packageIdentifier?.length ?: 0) < 4) {
                    println(red("[Error] Invalid Length - Length must be between 4 and 128 characters"))
                    println("\n")
                    println(brightGreen("[Required] Enter the Package Identifier, in the following format <Publisher shortname.Application shortname>. For example: Microsoft.Excel"))
                    packageIdentifier = prompt(brightWhite("Package Identifier"))?.trim()
                } else {
                    packageIdentifierSuccessful = true
                }
            }
        }
    }

    private fun packageVersionPrompt() {
        with(terminal) {
            println(brightGreen("[Required] Enter the version. for example: 1.33.7"))
            var packageVersion = prompt(brightWhite("Package Version"))?.trim()
        }
    }

}