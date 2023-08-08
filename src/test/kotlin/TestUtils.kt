import io.ktor.http.Url
import schemas.manifest.InstallerManifest

fun architectureUrl(architecture: String, delimiter: String = "-"): Url {
    return Url("https://www.example.com/file$delimiter$architecture${delimiter}extension")
}

fun architectureUrl(
    architecture: InstallerManifest.Installer.Architecture?,
    scope: InstallerManifest.Scope? = null,
    delimiter: String = "-"
): Url = architectureUrl(
    architecture = architecture,
    scope = scope,
    delimiterBefore = delimiter,
    delimiterAfter = delimiter
)

fun architectureUrl(
    architecture: InstallerManifest.Installer.Architecture?,
    scope: InstallerManifest.Scope? = null,
    delimiterBefore: String = "-",
    delimiterAfter: String = ".",
    extension: String = "exe"
): Url {
    return Url("https://www.example.com/file${scope ?: ""}$delimiterBefore${architecture ?: ""}${delimiterAfter}$extension")
}
