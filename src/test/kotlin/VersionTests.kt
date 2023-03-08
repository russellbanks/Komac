import data.shared.PackageVersion
import data.shared.PackageVersion.getError
import data.shared.PackageVersion.getHighestVersion
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe

class VersionTests : FunSpec({
    context("highest version number tests") {
        test("greater integer") {
            listOf("2", "1").getHighestVersion() shouldBe "2"
        }

        test("decimal point recognition") {
            listOf("1.0.1", "1.0", "1.0.0.1").getHighestVersion() shouldBe "1.0.1"
        }

        test("pre-release versions") {
            listOf("1.0.0-alpha", "1.0.0-beta").getHighestVersion() shouldBe "1.0.0-beta"
        }

        test("version number with 'v'") {
            listOf("v1", "1").getHighestVersion() shouldBe "1"
        }

        test("version number length") {
            listOf("13.0.0.8", "14", "14.0.1").getHighestVersion() shouldBe "14.0.1"
        }

        test("greater version with same pre-release word") {
            listOf("0.0.1-alpha", "0.0.2-alpha").getHighestVersion() shouldBe "0.0.2-alpha"
        }

        test("greater version with lesser pre-release word") {
            listOf("0.0.1-beta", "0.0.2-alpha").getHighestVersion() shouldBe "0.0.2-alpha"
        }


        test("minor version greater than or equal to 10") {
            listOf("1.9.0", "1.10.0").getHighestVersion() shouldBe "1.10.0"
        }

        test("long lesser number") {
            listOf("1.9.9.9.9.9.9", "2").getHighestVersion() shouldBe "2"
        }

        test("leading zeroes") {
            listOf("0000001", "2").getHighestVersion() shouldBe "2"
        }

        test("empty list") {
            listOf<String>().getHighestVersion() shouldBe null
        }

        test("largest version allowed") {
            val maxVersion = "9".repeat(PackageVersion.maxLength)
            listOf(maxVersion).getHighestVersion() shouldBe maxVersion
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
                it.getHighestVersion() shouldBe "2.0.0.1"
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
            getError(it) shouldBe null
        }

        test("version greater than max length fails") {
            getError("A".repeat(PackageVersion.maxLength.inc())) shouldNotBe null
        }

        test("blank or empty string fails") {
            getError(" ".repeat(10)) shouldNotBe null
        }

        test("invalid version pattern fails") {
            getError("£$%^&*()") shouldNotBe null
        }
    }
})
