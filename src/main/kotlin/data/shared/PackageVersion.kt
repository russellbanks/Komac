package data.shared

import commands.interfaces.TextPrompt
import commands.interfaces.ValidationRules

object PackageVersion : TextPrompt {
    override val name: String = "Package Version"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 128,
        minLength = 1,
        pattern = Regex("^[^\\\\/:*?\"<>|\\x01-\\x1f]+$"),
        isRequired = true
    )

    override val extraText: String = "Example: ${generateRandomVersion()}"

    /**
     * Generates a random version string in the format "major.minor.patch".
     *
     * @return a randomly generated version string.
     */
    private fun generateRandomVersion(): String {
        val major = (1 until 10).random()
        val minor = (0 until 100).random()
        val patch = (0 until 10).random()
        return "$major.$minor.$patch"
    }
}
