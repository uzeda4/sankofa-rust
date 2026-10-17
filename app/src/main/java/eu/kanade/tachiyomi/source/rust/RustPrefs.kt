package eu.kanade.tachiyomi.source.rust

import android.content.SharedPreferences
import androidx.preference.EditTextPreference
import androidx.preference.ListPreference
import androidx.preference.MultiSelectListPreference
import androidx.preference.PreferenceScreen
import androidx.preference.SwitchPreferenceCompat
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put

// KMK -->
/**
 * Settings a Rust source declares (`prefs` in its `info`): drawn as a standard preference screen, stored in the
 * source's own `SharedPreferences`, and sent back to the source with every request.
 */
internal object RustPrefs {

    fun setup(screen: PreferenceScreen, defs: JsonArray) {
        val context = screen.context
        defs.map { it.jsonObject }.forEach { d ->
            val key = d.str("key") ?: return@forEach
            val title = d.str("title").orEmpty()
            val summary = d.str("summary").orEmpty()
            val entries = (d["entries"] as? JsonArray).orEmpty().map { it.jsonArray }
            val labels = entries.map { (it[0] as JsonPrimitive).content }.toTypedArray()
            val values = entries.map { (it[1] as JsonPrimitive).content }.toTypedArray()
            val pref = when (d.str("type")) {
                "switch" -> SwitchPreferenceCompat(context).apply {
                    setDefaultValue((d["default"] as? JsonPrimitive)?.booleanOrNull ?: false)
                }
                "list" -> ListPreference(context).apply {
                    this.entries = labels
                    entryValues = values
                    setDefaultValue(d.str("default").orEmpty())
                    this.summary = summary.ifBlank { "%s" }
                }
                "multi_list" -> MultiSelectListPreference(context).apply {
                    this.entries = labels
                    entryValues = values
                    setDefaultValue((d["default"] as? JsonArray).orEmpty().map { (it as JsonPrimitive).content }.toSet())
                }
                "text" -> EditTextPreference(context).apply {
                    setDefaultValue(d.str("default").orEmpty())
                    this.summary = summary.ifBlank { "%s" }
                    setOnBindEditTextListener { it.setSingleLine() }
                }
                else -> return@forEach
            }
            pref.key = key
            pref.title = title
            if (pref !is ListPreference && pref !is EditTextPreference && summary.isNotBlank()) pref.summary = summary
            screen.addPreference(pref)
        }
    }

    /** Declared defaults overlaid with what the user stored, as the source reads them (`pref_*` in the SDK). */
    fun snapshot(stored: SharedPreferences, defs: JsonArray): JsonObject = buildJsonObject {
        defs.map { it.jsonObject }.forEach { d ->
            val key = d.str("key") ?: return@forEach
            val value: JsonElement = when (d.str("type")) {
                "switch" -> JsonPrimitive(
                    if (stored.contains(key)) stored.getBoolean(key, false) else (d["default"] as? JsonPrimitive)?.booleanOrNull ?: false,
                )
                "multi_list" -> {
                    val set = stored.getStringSet(key, null)
                    if (set != null) buildJsonArray { set.sorted().forEach { add(JsonPrimitive(it)) } } else d["default"] ?: JsonArray(emptyList())
                }
                else -> JsonPrimitive(stored.getString(key, null) ?: d.str("default").orEmpty())
            }
            put(key, value)
        }
    }

    private fun JsonObject.str(k: String): String? = (this[k] as? JsonPrimitive)?.contentOrNull
}
// KMK <--
