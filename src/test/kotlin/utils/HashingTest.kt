package utils

import com.appmattus.crypto.Algorithm
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.shouldBe
import utils.Hashing.hash

class HashingTest : FunSpec({
    context("SHA_256 hashes") {
        test("hash a blank string") {
            "".hash(Algorithm.SHA_256) shouldBe "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        }
    }

    context("SHA_1 hashes") {
        test("hash a blank string") {
            "".hash(Algorithm.SHA_1) shouldBe "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        }
    }

    context("XXH3_64 hashes") {
        test("hash a blank string") {
            "".hash(Algorithm.XXH3_64()) shouldBe "2d06800538d394c2"
        }
    }
})
