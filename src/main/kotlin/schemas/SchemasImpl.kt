package schemas

import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent

@Single
class SchemasImpl : KoinComponent {
    var manifestOverride: String? = null
}
