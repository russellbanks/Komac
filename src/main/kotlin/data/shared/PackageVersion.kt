package data.shared

import io.menu.prompts.TextPrompt
import io.menu.prompts.ValidationRules

object PackageVersion : TextPrompt {
    override val name: String = "Package Version"

    override val validationRules: ValidationRules = ValidationRules(
        maxLength = 128,
        minLength = 1,
        pattern = Regex("^[^\\\\/:*?\"<>|\\x01-\\x1f]+$"),
        isRequired = true
    )

    override val extraText: String = "Example: $randomVersion"

    /**
     * Generates a random version string in the format "major.minor.patch".
     *
     * @return a randomly generated version string.
     */
    private val randomVersion: String
        get() {
            val major = (1..<10).random()
            val minor = (0..<100).random()
            val patch = (0..<10).random()
            return "$major.$minor.$patch"
        }
}
