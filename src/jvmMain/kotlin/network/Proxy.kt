package network

actual object Proxy {
    private const val USE_SYSTEM_PROXIES = "java.net.useSystemProxies"
    actual fun useSystemProxy(): String? = System.setProperty(USE_SYSTEM_PROXIES, true.toString())
}
