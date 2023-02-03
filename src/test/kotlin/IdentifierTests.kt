import data.shared.PackageIdentifier
import data.shared.PackageIdentifier.getPackageIdentifierError
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe

class IdentifierTests : FunSpec({
    context("Package Identifier tests") {
        test("valid identifier is successful") {
            getPackageIdentifierError("Package.Identifier") shouldBe null
        }

        test("identifier greater than max segments fails") {
            getPackageIdentifierError(
                mutableListOf<String>().apply { repeat(9) { add("A") } }.joinToString(".")
            ) shouldNotBe null
        }

        test("identifier greater than max length fails") {
            getPackageIdentifierError("A".repeat(PackageIdentifier.maxLength.inc())) shouldNotBe null
        }

        test("identifier shorter than min length fails") {
            getPackageIdentifierError("A".repeat(PackageIdentifier.minLength.dec())) shouldNotBe null
        }

        test("blank or empty string fails") {
            getPackageIdentifierError(" ".repeat(10)) shouldNotBe null
        }

        test("identifier with only one segments fails") {
            getPackageIdentifierError("Identifier") shouldNotBe null
        }
    }
})
