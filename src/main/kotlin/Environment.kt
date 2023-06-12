import schemas.Schemas

object Environment {
    val isCI = System.getenv("CI")?.toBooleanStrictOrNull() == true
    val forkOverride = System.getenv(Schemas.customForkOwnerEnv)
}
