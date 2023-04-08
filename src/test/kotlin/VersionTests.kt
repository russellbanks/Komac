
import data.shared.PackageVersion
import extensions.versionStringComparator
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe

class VersionTests : FunSpec({
    context("highest version number tests") {
        test("greater integer") {
            maxOf("2", "1", versionStringComparator) shouldBe "2"
        }

        test("decimal point recognition") {
            maxOf("1.0.1", "1.0", "1.0.0.1", versionStringComparator) shouldBe "1.0.1"
        }

        test("pre-release versions") {
            maxOf("1.0.0-alpha", "1.0.0-beta", versionStringComparator) shouldBe "1.0.0-beta"
        }

        test("version number with 'v'") {
            maxOf("v1", "1", versionStringComparator) shouldBe "1"
        }

        test("version number length") {
            maxOf("13.0.0.8", "14", "14.0.1", versionStringComparator) shouldBe "14.0.1"
        }

        test("greater version with same pre-release word") {
            maxOf("0.0.1-alpha", "0.0.2-alpha", versionStringComparator) shouldBe "0.0.2-alpha"
        }

        test("greater version with lesser pre-release word") {
            maxOf("0.0.1-beta", "0.0.2-alpha", versionStringComparator) shouldBe "0.0.2-alpha"
        }

        test("minor version greater than or equal to 10") {
            maxOf("1.9.0", "1.10.0", versionStringComparator) shouldBe "1.10.0"
        }

        test("long lesser number") {
            maxOf("1.9.9.9.9.9.9", "2", versionStringComparator) shouldBe "2"
        }

        test("leading zeroes") {
            maxOf("0000001", "2", versionStringComparator) shouldBe "2"
        }

        test("empty list") {
            listOf<String>().maxWithOrNull(versionStringComparator) shouldBe null
        }

        test("largest version allowed") {
            val maxVersion = "9".repeat(PackageVersion.validationRules.maxLength as Int)
            listOf(maxVersion).maxWithOrNull(versionStringComparator) shouldBe maxVersion
        }

        context("every combination of a list's order should return the same value") {
            fun <T> permute(input: List<T>): List<List<T>> {
                if (input.size == 1) return listOf(input)
                val output = mutableListOf<List<T>>()
                for (i in input.indices) {
                    val element = input[i]
                    val remaining = input.filterIndexed { index, _ -> index != i }
                    for (perm in permute(remaining)) {
                        output.add(listOf(element) + perm)
                    }
                }
                return output
            }

            withData(permute(listOf("1.0.0", "2.0.0.1", "v1.0.0-alpha"))) {
                it.maxWithOrNull(versionStringComparator) shouldBe "2.0.0.1"
            }
        }
    }

    context("package version validity tests") {
        withData(
            "1.0.0",
            "v1",
            "123",
            "14.0.1",
            "1.10"
        ) {
            PackageVersion.getError(it) shouldBe null
        }

        test("version greater than max length fails") {
            PackageVersion.getError("A".repeat(PackageVersion.validationRules.maxLength?.inc() as Int)) shouldNotBe null
        }

        test("blank or empty string fails") {
            PackageVersion.getError(" ".repeat(10)) shouldNotBe null
        }

        test("invalid version pattern fails") {
            PackageVersion.getError("£$%^&*()") shouldNotBe null
        }
    }
})
