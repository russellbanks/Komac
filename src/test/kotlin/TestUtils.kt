import io.ktor.http.Url
import schemas.manifest.InstallerManifest

fun architectureUrl(architecture: String, delimiter: String = "-"): Url {
    return Url("https://www.example.com/file$delimiter$architecture${delimiter}extension")
}

fun architectureUrl(
    architecture: InstallerManifest.Installer.Architecture?,
    scope: InstallerManifest.Scope? = null,
    delimiter: String = "-"
): Url {
    return Url("https://www.example.com/file${scope ?: ""}$delimiter${architecture ?: ""}${delimiter}extension")
}
