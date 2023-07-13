package network

object Proxy {
    private const val USE_SYSTEM_PROXIES = "java.net.useSystemProxies"
    fun useSystemProxy(): String? = System.setProperty(USE_SYSTEM_PROXIES, true.toString())
}
