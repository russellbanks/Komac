
import extensions.YamlExtensions.convertToList
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe

class YamlTests : FunSpec({
    context("convert input to a yaml list") {
        test("convert string to list by ', '") {
            "tag1, tag2, tag3".convertToList() shouldBe listOf("tag1", "tag2", "tag3")
        }

        test("convert string to list by ','") {
            "tag1,tag2,tag3".convertToList() shouldBe listOf("tag1", "tag2", "tag3")
        }

        test("convert string to list by a space") {
            "tag1 tag2 tag3".convertToList() shouldBe listOf("tag1", "tag2", "tag3")
        }

        test("duplicate items are not included in the list") {
            "tag1, tag2, tag3, tag2".convertToList() shouldBe listOf("tag1", "tag2", "tag3")
        }

        test("duplicate items are included in the list when uniqueItems is false") {
            "tag1, tag2, tag3, tag2".convertToList(uniqueItems = false) shouldBe listOf(
                "tag1",
                "tag2",
                "tag2",
                "tag3"
            )
        }

        test("items are sorted in the list") {
            "tag4, tag2, tag3, tag1".convertToList() shouldBe listOf("tag1", "tag2", "tag3", "tag4")
        }

        test("multiple delimiters can be used in the string") {
            "tag1 tag2, tag3#tag4".convertToList() shouldBe listOf("tag1", "tag2", "tag3", "tag4")
        }
    }
})
