import data.shared.PackageVersion
import data.shared.PackageVersion.getHighestVersion
import data.shared.PackageVersion.getPackageVersionError
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe

class VersionTests : FunSpec({
    context("highest version number tests") {
        test("greater integer") {
            getHighestVersion(listOf("2", "1")) shouldBe "2"
        }
        test("decimal point recognition") {
            getHighestVersion(listOf("1.0.1", "1.0", "1.0.0.1")) shouldBe "1.0.1"
        }
        test("pre-release versions") {
            getHighestVersion(listOf("1.0.0-alpha", "1.0.0-beta")) shouldBe "1.0.0-beta"
        }
        test("version number with 'v'") {
            getHighestVersion(listOf("v1", "1")) shouldBe "1"
        }
        test("version number length") {
            getHighestVersion(listOf("13.0.0.8", "14", "14.0.1")) shouldBe "14.0.1"
        }
        test("greater version with same pre-release word") {
            getHighestVersion(listOf("0.0.1-alpha", "0.0.2-alpha")) shouldBe "0.0.2-alpha"
        }
        test("greater version with lesser pre-release word") {
            getHighestVersion(listOf("0.0.1-beta", "0.0.2-alpha")) shouldBe "0.0.2-alpha"
        }
        test("minor version greater than or equal to 10") {
            getHighestVersion(listOf("1.9.0", "1.10.0")) shouldBe "1.10.0"
        }
        test("long lesser number") {
            getHighestVersion(listOf("1.9.9.9.9.9.9", "2")) shouldBe "2"
        }
        test("leading zeroes") {
            getHighestVersion(listOf("0000001", "2")) shouldBe "2"
        }
        context("every combination of a list's order should return the same value") {
            fun permutations(list: List<String>): List<List<String>> {
                if (list.size == 1) return listOf(list)
                val result = mutableListOf<List<String>>()
                for (i in list.indices) {
                    val subList = list.toMutableList().apply { removeAt(i) }
                    permutations(subList).forEach { result.add(listOf(list[i]) + it) }
                }
                return result
            }

            withData(permutations(listOf("1.0.0", "2.0.0.1", "v1.0.0-alpha"))) {
                getHighestVersion(it) shouldBe "2.0.0.1"
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
            getPackageVersionError(it) shouldBe null
        }

        test("version greater than max length fails") {
            getPackageVersionError("A".repeat(PackageVersion.maxLength.inc())) shouldNotBe null
        }

        test("blank or empty string fails") {
            getPackageVersionError(" ".repeat(10)) shouldNotBe null
        }

        test("invalid version pattern fails") {
            getPackageVersionError("£$%^&*()") shouldNotBe null
        }
    }
})
