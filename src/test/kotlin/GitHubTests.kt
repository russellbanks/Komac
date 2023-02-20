import detection.github.GitHubExtensions.getFormattedReleaseNotes
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import io.mockk.every
import io.mockk.mockk
import org.kohsuke.github.GHRelease

class GitHubTests : FunSpec({
    context("formatted release notes tests") {
        test("format title and bullet point") {
            val ghRelease: GHRelease = mockk {
                every { body } returns """
                    ## Title
                    
                    - Bullet point 1
                """.trimIndent()
            }
            getFormattedReleaseNotes(ghRelease) shouldBe """
                Title
                - Bullet point 1
            """.trimIndent()
        }

        test("single title returns null") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "# Title"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe null
        }

        test("asterisk bullet points are converted to dashes") {
            val ghRelease: GHRelease = mockk {
                every { body } returns """
                    # Title
                    * Bullet 1
                    * Bullet 2
                """.trimIndent()
            }
            getFormattedReleaseNotes(ghRelease) shouldBe """
                Title
                - Bullet 1
                - Bullet 2
            """.trimIndent()
        }

        test("formatting on bold text is removed") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- **Bold**"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- Bold"
        }

        test("formatting on code is removed") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- `Code here`"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- Code here"
        }

        test("formatting on strikethrough text is removed") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- ~Strikethrough~ ~~~Strikethrough text 2~~~"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- Strikethrough Strikethrough text 2"
        }

        test("dropdowns are removed") {
            val ghRelease: GHRelease = mockk {
                every { body } returns """
                    <details>
                        <summary>Dropdown title</summary>
                    </details>
                    - Bullet point
                """.trimIndent()
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- Bullet point"
        }

        test("titles without a bullet point within two lines aren't included") {
            val ghRelease: GHRelease = mockk {
                every { body } returns """
                    # Title
                    
                    
                    - Bullet point
                """.trimIndent()
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- Bullet point"
        }

        test("headers have # removed") {
            val ghRelease: GHRelease = mockk {
                every { body } returns """
                    #### Header
                    - Bullet point
                """.trimIndent()
            }
            getFormattedReleaseNotes(ghRelease) shouldBe """
                Header
                - Bullet point
            """.trimIndent()
        }

        test("markdown links are converted into plaintext") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- [Text](Link)"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- Text"
        }

        test("bullet points with several sentences are split onto new lines and indented") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- First sentence. Second sentence. Third sentence."
            }
            getFormattedReleaseNotes(ghRelease) shouldBe """
                - First sentence.
                  Second sentence.
                  Third sentence.
            """.trimIndent()
        }

        test("lines without a space after their bullet point are not included") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "-Sentence"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe null
        }

        test("null release notes return null") {
            val ghRelease: GHRelease = mockk {
                every { body } returns null
            }
            getFormattedReleaseNotes(ghRelease) shouldBe null
        }

        test("blank release notes return null") {
            val ghRelease: GHRelease = mockk {
                every { body } returns " ".repeat(10)
            }
            getFormattedReleaseNotes(ghRelease) shouldBe null
        }

        test("lines that have miscellaneous html tags are not included") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "<html> </html>"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe null
        }

        test("empty bullet points are not included") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- "
            }
            getFormattedReleaseNotes(ghRelease) shouldBe null
        }

        test("images get removed") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- ![Alt text](image link)"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe null
        }

        test("linked images get removed") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- [![Alt text](image link)](link)"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe null
        }

        test("pull request links are converted to their pull request number") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- New feature in https://github.com/user/repository/pull/1234"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- New feature in #1234"
        }

        test("issue links are converted to their issue number") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- Issue reported in https://github.com/user/repository/issues/4321"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- Issue reported in #4321"
        }

        test("multiple pull request or issue links in a string are converted to their pull request numbers") {
            val ghRelease: GHRelease = mockk {
                every { body } returns buildString {
                    append("- New features in ")
                    append("https://github.com/user/repository/issues/1234")
                    append(" and ")
                    append("https://github.com/user/repository/pull/4321")
                }
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- New features in #1234 and #4321"
        }

        test("pull requests without a number don't get converted") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- https://github.com/user/repository/pull"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- https://github.com/user/repository/pull"
        }

        test("issues without a number don't get converted") {
            val ghRelease: GHRelease = mockk {
                every { body } returns "- https://github.com/user/repository/issues"
            }
            getFormattedReleaseNotes(ghRelease) shouldBe "- https://github.com/user/repository/issues"
        }
    }
})
