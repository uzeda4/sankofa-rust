package eu.kanade.tachiyomi.source.rust

import eu.kanade.tachiyomi.source.model.Filter
import eu.kanade.tachiyomi.source.model.FilterList
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put

// KMK -->
/**
 * Search filters of a Rust source. The source describes them as JSON (`filters` in `docs/redesign/RUST-SOURCES.md`);
 * each one keeps its `key` so the picks go back to the source as `{key: state}`.
 */
internal object RustFilters {

    fun build(defs: JsonArray): FilterList = FilterList(defs.mapNotNull { it.jsonObject.toFilter() })

    /** The picks as the source reads them (`Filters` in the SDK): key to state, groups nested. */
    fun state(filters: List<Filter<*>>): JsonObject = buildJsonObject {
        filters.forEach { f ->
            when (f) {
                is Text -> put(f.key, f.state)
                is Select -> put(f.key, f.state)
                is Check -> put(f.key, f.state)
                is Tri -> put(f.key, f.state)
                is Group -> put(f.key, state(f.state))
                is SortBy -> f.state?.let { s ->
                    put(
                        f.key,
                        buildJsonObject {
                            put("index", s.index)
                            put("ascending", s.ascending)
                        },
                    )
                }
                else -> Unit
            }
        }
    }

    private fun JsonObject.toFilter(): Filter<*>? {
        val key = str("key").orEmpty()
        val name = str("name").orEmpty()
        return when (str("type")) {
            "header" -> Filter.Header(name)
            "separator" -> Filter.Separator()
            "text" -> Text(key, name, str("default").orEmpty())
            "select" -> Select(key, name, labels(this["options"]), (this["default"] as? JsonPrimitive)?.intOrNull ?: 0)
            "checkbox" -> Check(key, name, (this["default"] as? JsonPrimitive)?.booleanOrNull ?: false)
            "tristate" -> Tri(key, name)
            "group" -> Group(key, name, (this["items"] as? JsonArray).orEmpty().mapNotNull { it.jsonObject.toFilter() })
            "sort" -> SortBy(
                key,
                name,
                labels(this["options"]),
                Filter.Sort.Selection(
                    (this["default_index"] as? JsonPrimitive)?.intOrNull ?: 0,
                    (this["default_ascending"] as? JsonPrimitive)?.booleanOrNull ?: false,
                ),
            )
            else -> null
        }
    }

    private fun labels(e: JsonElement?): Array<String> =
        (e as? JsonArray).orEmpty().map { (it as JsonPrimitive).content }.toTypedArray()

    private fun JsonObject.str(k: String): String? = (this[k] as? JsonPrimitive)?.contentOrNull

    class Text(val key: String, name: String, state: String) : Filter.Text(name, state)
    class Select(val key: String, name: String, values: Array<String>, state: Int) : Filter.Select<String>(name, values, state)
    class Check(val key: String, name: String, state: Boolean) : Filter.CheckBox(name, state)
    class Tri(val key: String, name: String) : Filter.TriState(name)
    class Group(val key: String, name: String, items: List<Filter<*>>) : Filter.Group<Filter<*>>(name, items)
    class SortBy(val key: String, name: String, values: Array<String>, state: Selection?) : Filter.Sort(name, values, state)
}
// KMK <--
