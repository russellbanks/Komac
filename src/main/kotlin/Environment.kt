import schemas.Schemas

object Environment {
    val isCI = System.getenv("CI")?.toBooleanStrictOrNull() == true
    val forkOverride: String? = System.getenv(Schemas.customForkOwnerEnv)
}
