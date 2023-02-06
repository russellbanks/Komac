import data.shared.PackageIdentifier
import data.shared.PackageIdentifier.getError
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe

class IdentifierTests : FunSpec({
    context("package identifier validity tests") {
        test("valid identifier is successful") {
            getError("Package.Identifier") shouldBe null
        }

        test("identifier greater than max segments fails") {
            getError(
                mutableListOf<String>().apply { repeat(9) { add("A") } }.joinToString(".")
            ) shouldNotBe null
        }

        test("identifier greater than max length fails") {
            getError("A".repeat(PackageIdentifier.maxLength.inc())) shouldNotBe null
        }

        test("identifier shorter than min length fails") {
            getError("A".repeat(PackageIdentifier.minLength.dec())) shouldNotBe null
        }

        test("blank or empty string fails") {
            getError(" ".repeat(10)) shouldNotBe null
        }

        test("identifier with only one segments fails") {
            getError("Identifier") shouldNotBe null
        }
    }
})
