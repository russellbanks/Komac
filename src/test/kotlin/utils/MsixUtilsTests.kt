package utils

import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe

class MsixUtilsTests : FunSpec({
    context("calculate package family name") {
        test("identity name and full identity publisher") {
            val identityName = "PackageName"
            val identityPublisher = listOf(
                "CN=Publisher Software",
                "O=Publisher Software",
                "L=Zürich",
                "S=Zürich",
                "C=CH"
            ).joinToString()
            MsixUtils.getPackageFamilyName(identityName, identityPublisher) shouldBe "PackageName_31kpdnra495ry"
        }

        test("empty identity name and identity publisher") {
            val identityName = ""
            val identityPublisher = ""
            MsixUtils.getPackageFamilyName(identityName, identityPublisher) shouldBe "_werc8gmrzge18"
        }

        test("identity name containing underscore and identity publisher with non-ASCII characters") {
            val identityName = "my_app_name"
            val identityPublisher = "Publisher Software Göteborg"
            MsixUtils.getPackageFamilyName(identityName, identityPublisher) shouldBe "my_app_name_0cddhv5z1eydp"
        }

        test("identity name and identity publisher containing only ASCII characters") {
            val identityName = "AppName"
            val identityPublisher = "Publisher Software"
            MsixUtils.getPackageFamilyName(identityName, identityPublisher) shouldBe "AppName_zj75k085cmj1a"
        }

        test("identity name and identity publisher containing special characters") {
            val identityName = "My App-Name_1.0"
            val identityPublisher = "Publisher (Software)"
            MsixUtils.getPackageFamilyName(identityName, identityPublisher) shouldBe "My App-Name_1.0_xz34dc2mhee0e"
        }

        test("identity name and identity publisher containing only non-alphanumeric characters") {
            val identityName = "!@#$%^&*()"
            val identityPublisher = "!@#$%^&*()"
            MsixUtils.getPackageFamilyName(identityName, identityPublisher) shouldBe "!@#$%^&*()_3twee7s7mwzyy"
        }

        test("identity name and identity publisher containing only numeric characters") {
            val identityName = "12345"
            val identityPublisher = "67890"
            MsixUtils.getPackageFamilyName(identityName, identityPublisher) shouldBe "12345_pydnmasy62pxp"
        }

        test("identity name and identity publisher containing only whitespace characters") {
            val identityName = "   "
            val identityPublisher = "   "
            MsixUtils.getPackageFamilyName(identityName, identityPublisher) shouldBe "   _2bkd433cae3jj"
        }

        test("identity publisher that is very long") {
            val identityName = "x"
            val identityPublisher = "y".repeat(1024)
            MsixUtils.getPackageFamilyName(identityName, identityPublisher) shouldBe  "x_r0rvk7mj3mn82"
        }
    }
})
