import data.shared.PackageIdentifier
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe

class IdentifierTests : FunSpec({
    context("package identifier validity tests") {
        test("valid identifier is successful") {
            PackageIdentifier.getError("Package.Identifier") shouldBe null
        }

        test("identifier greater than max segments fails") {
            PackageIdentifier.getError(
                mutableListOf<String>().apply { repeat(9) { add("A") } }.joinToString(".")
            ) shouldNotBe null
        }

        test("identifier greater than max length fails") {
            PackageIdentifier.getError(
                "A".repeat(PackageIdentifier.validationRules.maxLength?.inc() as Int)
            ) shouldNotBe null
        }

        test("identifier shorter than min length fails") {
            PackageIdentifier.getError(
                "A".repeat(PackageIdentifier.validationRules.minLength?.dec() as Int)
            ) shouldNotBe null
        }

        test("blank or empty string fails") {
            PackageIdentifier.getError(" ".repeat(10)) shouldNotBe null
        }

        test("identifier with only one segments fails") {
            PackageIdentifier.getError("Identifier") shouldNotBe null
        }
    }
})
