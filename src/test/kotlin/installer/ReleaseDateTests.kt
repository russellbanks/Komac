package installer

import Validation
import data.ReleaseDate.isReleaseDateValid
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import schemas.Pattern
import java.text.SimpleDateFormat
import java.util.Date
import kotlin.random.Random

class ReleaseDateTests : FunSpec({
    context("Release Date Tests") {
        withData(
            List(50) {
                SimpleDateFormat(Pattern.releaseDate).format(Date((Random.nextDouble() * Date().time).toLong()))
            }
        ) {
            isReleaseDateValid(it).first shouldBe Validation.Success
        }

        withData(
            List(50) {
                SimpleDateFormat("yyyy-MM-dd")
                    .format(Date((Math.random() * Date().time).toLong()))
                    .split("-")
                    .reversed()
                    .joinToString("-")
            }
        ) {
            isReleaseDateValid(it).first shouldNotBe Validation.Success
        }
    }
})
