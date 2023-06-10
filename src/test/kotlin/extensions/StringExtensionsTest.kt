package extensions

import github.ReleaseNotesFormatter.cutToCharLimitWithLines
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe

class StringExtensionsTest : FunSpec({
    test("limitLinesToCharLimit returns same string when limit is larger than string length") {
        val testString = "Hello, world!"
        testString.cutToCharLimitWithLines(50) shouldBe testString
    }

    test("limitLinesToCharLimit limits string to specified char limit") {
        val testString = """
            Hello
            world!
            This is a test string
        """.trimIndent()
        val expected = """
            Hello
            world!
        """.trimIndent()
        testString.cutToCharLimitWithLines(12) shouldBe expected
    }

    test("limitLinesToCharLimit returns empty string when limit is 0") {
        val testString = "Hello, world!"
        testString.cutToCharLimitWithLines(0) shouldBe ""
    }
})
