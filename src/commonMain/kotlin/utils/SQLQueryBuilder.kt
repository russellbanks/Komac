package utils

class SQLQueryBuilder {
    private var select: String = ""
    private var from: String = ""
    private var where: String = ""

    fun select(vararg columns: String) {
        select = "SELECT ${columns.joinToString(", ") { "`$it`" }}"
    }

    fun from(table: String) {
        from = "FROM `$table`"
    }

    fun where(property: String, values: List<String>) {
        where = "WHERE ${values.joinToString(" OR ") { "`$property` = '$it'" }}"
    }

    fun build(): String {
        require(select.isNotEmpty()) { "SELECT clause cannot be empty" }
        require(from.isNotEmpty()) { "FROM clause cannot be empty" }

        return "$select $from${if (where.isNotEmpty()) " $where" else ""}"
    }
}

fun sqlQuery(init: SQLQueryBuilder.() -> Unit): String = SQLQueryBuilder().apply(init).build()
