object Errors {
    fun invalidLength(min: Int, max: Int): String {
        return "[Error] Invalid Length - Length must be between $min and $max characters"
    }
    const val invalidRegex = "[Error] Invalid Pattern - The value entered does not match the pattern requirements defined in the manifest schema"

    const val genericError = "[Internal Error] - Value was not able to be saved successfully"

    const val unsuccessfulUrlResponse = "[Error] Invalid URL Response - The URL did not return a successful response from the server"
}
