package detection.files.msi

class SQLBuilder {
    private val statement = StringBuilder()
    fun select(vararg values: String): SQLBuilder {
        statement.append("SELECT ")
        values.forEachIndexed { index, value ->
            statement.append("`$value`")
            if (index < values.size - 1) {
                statement.append(", ")
            }
        }
        return this
    }

    fun from(table: String): SQLBuilder {
        statement.append(" FROM `$table`")
        return this
    }

    fun where(property: String, values: List<String>): SQLBuilder {
        statement.append(" WHERE ")
        values.forEachIndexed { index, value ->
            statement.append("`$property` = '$value'")
            if (index < values.size - 1) {
                statement.append(" OR ")
            }
        }
        return this
    }

    override fun toString(): String {
        return statement.toString()
    }
}
