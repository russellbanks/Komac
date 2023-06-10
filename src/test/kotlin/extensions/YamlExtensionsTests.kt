package extensions
import schemas.manifest.YamlExtensions.convertToList
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe

class YamlExtensionsTests : FunSpec({
    context("convert input to a yaml list") {
        test("convert string to list by ', '") {
            convertToList("tag1, tag2, tag3") shouldBe listOf("tag1", "tag2", "tag3")
        }

        test("convert string to list by ','") {
            convertToList("tag1,tag2,tag3") shouldBe listOf("tag1", "tag2", "tag3")
        }

        test("convert string to list by a space") {
            convertToList("tag1 tag2 tag3") shouldBe listOf("tag1", "tag2", "tag3")
        }

        test("duplicate items are not included in the list") {
            convertToList("tag1, tag2, tag3, tag2") shouldBe listOf("tag1", "tag2", "tag3")
        }

        test("items are sorted in the list") {
            convertToList("tag4, tag2, tag3, tag1") shouldBe listOf("tag1", "tag2", "tag3", "tag4")
        }

        test("multiple delimiters can be used in the string") {
            convertToList("tag1 tag2, tag3#tag4") shouldBe listOf("tag1", "tag2", "tag3", "tag4")
        }
    }
})
