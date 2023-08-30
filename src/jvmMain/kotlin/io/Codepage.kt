package io

import com.sun.jna.platform.win32.Kernel32
import java.io.PrintStream

actual object Codepage {
    private const val UTF8 = 65001

    private val initialCodePage = Kernel32.INSTANCE.GetConsoleCP()
    private val initialOutputCodePage = Kernel32.INSTANCE.GetConsoleOutputCP()

    actual fun setConsoleUTF8() = Kernel32.INSTANCE.run {
        if (initialCodePage != UTF8) {
            SetConsoleCP(UTF8)
        }
        if (initialOutputCodePage != UTF8) {
            SetConsoleOutputCP(UTF8)
            // Refresh the cached Standard Output now we have a new codepage
            System.setOut(PrintStream(System.out, true, Charsets.UTF_8))
        }
        Runtime.getRuntime().addShutdownHook(Thread(Codepage::resetCodepage))
    }

    actual fun resetCodepage() = Kernel32.INSTANCE.run {
        if (GetConsoleCP() != initialCodePage) {
            SetConsoleCP(initialCodePage)
        }
        if (GetConsoleOutputCP() != initialOutputCodePage) {
            SetConsoleOutputCP(initialOutputCodePage)
        }
    }
}