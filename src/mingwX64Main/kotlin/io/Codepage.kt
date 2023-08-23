package io

import platform.windows.GetConsoleCP
import platform.windows.GetConsoleOutputCP
import platform.windows.SetConsoleCP
import platform.windows.SetConsoleOutputCP

actual object Codepage {
    private const val UTF8 = 65001U
    private val initialCodePage = GetConsoleCP()
    private val initialOutputCodePage = GetConsoleOutputCP()

    actual fun setConsoleUTF8() {
        if (initialCodePage != UTF8) {
            SetConsoleCP(UTF8)
        }
        if (initialOutputCodePage != UTF8) {
            SetConsoleOutputCP(UTF8)
        }
    }
    
    actual fun resetCodepage() {
        if (GetConsoleCP() != initialCodePage) {
            SetConsoleCP(initialCodePage)
        }
        if (GetConsoleOutputCP() != initialOutputCodePage) {
            SetConsoleOutputCP(initialOutputCodePage)
        }
    }
}
