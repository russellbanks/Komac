
import extensions.formattedReleaseNotes
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.mockk.every
import io.mockk.mockk
import okio.ByteString.Companion.encodeUtf8
import org.kohsuke.github.GHRelease
import org.kohsuke.github.GHRepository

class GitHubTests : FunSpec({
    context("formatted release notes tests") {
        val repository: GHRepository = mockk {
            every { fullName } returns "user/repository"
        }
        val ghRelease: GHRelease = mockk {
            every { owner } returns repository
        }

        test("format title and bullet point") {
            every { ghRelease.body } returns """
                ## Title
                
                - Bullet point 1
            """.trimIndent()
            ghRelease.formattedReleaseNotes shouldBe """
                Title
                - Bullet point 1
            """.trimIndent()
        }

        test("single title returns null") {
            every { ghRelease.body } returns "# Title"
            ghRelease.formattedReleaseNotes shouldBe null
        }

        test("asterisk bullet points are converted to dashes") {
            every { ghRelease.body } returns """
                # Title
                * Bullet 1
                * Bullet 2
            """.trimIndent()
            ghRelease.formattedReleaseNotes shouldBe """
                Title
                - Bullet 1
                - Bullet 2
            """.trimIndent()
        }

        test("formatting on bold text is removed") {
            every { ghRelease.body } returns "- **Bold**"
            ghRelease.formattedReleaseNotes shouldBe "- Bold"
        }

        test("formatting on code is removed") {
            every { ghRelease.body } returns "- `Code here`"
            ghRelease.formattedReleaseNotes shouldBe "- Code here"
        }

        test("formatting on strikethrough text is removed") {
            every { ghRelease.body } returns "- ~Strikethrough~ ~~~Strikethrough text 2~~~"
            ghRelease.formattedReleaseNotes shouldBe "- Strikethrough Strikethrough text 2"
        }

        test("dropdowns are removed") {
            every { ghRelease.body } returns """
                <details>
                    <summary>Dropdown title</summary>
                </details>
                - Bullet point
            """.trimIndent()
            ghRelease.formattedReleaseNotes shouldBe "- Bullet point"
        }

        test("titles without a bullet point within two lines aren't included") {
            every { ghRelease.body } returns """
                # Title
                
                
                - Bullet point
            """.trimIndent()
            ghRelease.formattedReleaseNotes shouldBe "- Bullet point"
        }

        test("headers have # removed") {
            every { ghRelease.body } returns """
                #### Header
                - Bullet point
            """.trimIndent()
            ghRelease.formattedReleaseNotes shouldBe """
                Header
                - Bullet point
            """.trimIndent()
        }

        test("markdown links are converted into plaintext") {
            every { ghRelease.body } returns "- [Text](Link)"
            ghRelease.formattedReleaseNotes shouldBe "- Text"
        }

        test("bullet points with several sentences are split onto new lines and indented") {
            every { ghRelease.body } returns "- First sentence. Second sentence. Third sentence."
            ghRelease.formattedReleaseNotes shouldBe """
                - First sentence.
                  Second sentence.
                  Third sentence.
            """.trimIndent()
        }

        test("lines without a space after their bullet point are not included") {
            every { ghRelease.body } returns "-Sentence"
            ghRelease.formattedReleaseNotes shouldBe null
        }

        test("null release notes return null") {
            every { ghRelease.body } returns null
            ghRelease.formattedReleaseNotes shouldBe null
        }

        test("blank release notes return null") {
            every { ghRelease.body } returns " ".repeat(10)
            ghRelease.formattedReleaseNotes shouldBe null
        }

        test("lines that have miscellaneous html tags are not included") {
            every { ghRelease.body } returns "<html> </html>"
            ghRelease.formattedReleaseNotes shouldBe null
        }

        test("empty bullet points are not included") {
            every { ghRelease.body } returns "- "
            ghRelease.formattedReleaseNotes shouldBe null
        }

        test("images get removed") {
            every { ghRelease.body } returns "- ![Alt text](image link)"
            ghRelease.formattedReleaseNotes shouldBe null
        }

        test("linked images get removed") {
            every { ghRelease.body } returns "- [![Alt text](image link)](link)"
            ghRelease.formattedReleaseNotes shouldBe null
        }

        test("pull request links are converted to their pull request number") {
            every { ghRelease.body } returns "- New feature in https://github.com/user/repository/pull/1234"
            ghRelease.formattedReleaseNotes shouldBe "- New feature in #1234"
        }

        test("issue links are converted to their issue number") {
            every { ghRelease.body } returns "- Issue reported in https://github.com/user/repository/issues/4321"
            ghRelease.formattedReleaseNotes shouldBe "- Issue reported in #4321"
        }

        test("multiple pull request or issue links in a string are converted to their issue numbers") {
            every { ghRelease.body } returns buildString {
                append("- New features in ")
                append("https://github.com/user/repository/issues/1234")
                append(" and ")
                append("https://github.com/user/repository/pull/4321")
            }
            ghRelease.formattedReleaseNotes shouldBe "- New features in #1234 and #4321"
        }

        test("issues with a dash in the user or repository are converted to their issue numbers") {
            every { ghRelease.body } returns "- https://github.com/user-name/repository-extra/issues/1234"
            every { repository.fullName } returns "user-name/repository-extra"
            ghRelease.formattedReleaseNotes shouldBe "- #1234"
        }

        test("pull requests with a dash in the user or repository are converted to their issue numbers") {
            every { ghRelease.body } returns "- https://github.com/user-name/repository-extra/pull/4321"
            every { repository.fullName } returns "user-name/repository-extra"
            ghRelease.formattedReleaseNotes shouldBe "- #4321"
        }

        test("pull requests without a number don't get converted") {
            every { ghRelease.body } returns "- https://github.com/user/repository/pull"
            ghRelease.formattedReleaseNotes shouldBe "- https://github.com/user/repository/pull"
        }

        test("issues without a number don't get converted") {
            every { ghRelease.body } returns "- https://github.com/user/repository/issues"
            ghRelease.formattedReleaseNotes shouldBe "- https://github.com/user/repository/issues"
        }

        test("issues outside the repository are converted appropriately") {
            every { ghRelease.body } returns "- https://github.com/other-user/repository/issues/1234"
            ghRelease.formattedReleaseNotes shouldBe "- other-user/repository#1234"
        }

        test("pull requests outside the repository are converted appropriately") {
            every { ghRelease.body } returns "- https://github.com/user/other-repository/pull/4321"
            ghRelease.formattedReleaseNotes shouldBe "- user/other-repository#4321"
        }

        test("commit SHA-1's are removed") {
            val sha1 = "".encodeUtf8().sha1().hex()
            every { ghRelease.body } returns "- $sha1 New feature"
            ghRelease.formattedReleaseNotes shouldBe "- New feature"
        }

        test("amount of indentation is correct when there are multiple headers") {
            every { ghRelease.body } returns """
                # Header 1
                * Changes in Header 1
                # Header 2
                * Changes in Header 2
            """.trimIndent()
            ghRelease.formattedReleaseNotes shouldBe """
                Header 1
                - Changes in Header 1
                Header 2
                - Changes in Header 2
            """.trimIndent()
        }

        test("GitHub emojis in the format of :emoji: are replaced with actual emojis") {
            every { ghRelease.body } returns """
                # :wrench: Big changes :sparkles:
                - :100: Bullet point :+1:
            """.trimIndent()
            ghRelease.formattedReleaseNotes shouldBe """
                🔧 Big changes ✨
                - 💯 Bullet point 👍
            """.trimIndent()
        }

        test("Emojis that are already in unicode stay as they are") {
            every { ghRelease.body } returns """
                # 🔧 Big changes ✨
                * 💯 Bullet point 👍
            """.trimIndent()
            ghRelease.formattedReleaseNotes shouldBe """
                🔧 Big changes ✨
                - 💯 Bullet point 👍
            """.trimIndent()
        }
    }
})
