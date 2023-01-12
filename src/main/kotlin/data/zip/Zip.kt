package data.zip

import schemas.InstallerManifest
import java.io.File
import java.util.zip.ZipFile

class Zip(zip: File) {
    val nestedInstallerFiles: InstallerManifest.NestedInstallerFiles? = null
    init {
        require(zip.extension.lowercase() == InstallerManifest.InstallerType.ZIP.toString()) {
            "File must be a ${InstallerManifest.InstallerType.ZIP}"
        }
        ZipFile(zip).use { zip ->
            val entries = zip.entries()
            val exes = mutableListOf<String>()
            while (entries.hasMoreElements()) {
                val entry = entries.nextElement()
                val extension = entry.name.substringAfterLast(".")
                if (extension == "exe") {
                    exes.add(entry.name)
                }
            }
            exes.forEach(::println)
        }
    }
}
