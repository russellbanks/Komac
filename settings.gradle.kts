rootProject.name = "Komac"

pluginManagement {
    repositories {
        gradlePluginPortal()
        maven("https://maven.hq.hydraulic.software")
    }
}

dependencyResolutionManagement {
    versionCatalogs {
        create("libs") {
            from(files("libs.versions.toml"))
        }
    }
}
