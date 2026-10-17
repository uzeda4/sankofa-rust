package eu.kanade.tachiyomi.source.rust

import android.util.Base64
import eu.kanade.tachiyomi.network.NetworkHelper
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import uniffi.sk_api.HttpBridge
import uy.kohesive.injekt.Injekt
import uy.kohesive.injekt.api.get

// KMK -->
/**
 * The network of the sources written in Rust: their requests run on the app's own OkHttp client, so Cloudflare
 * handling, cookies, the user agent the user chose and the source's rate limit apply exactly as for extensions.
 */
object RustHttpBridge : HttpBridge {

    override fun cookie(sourceId: Long, url: String, name: String): String = runCatching {
        Injekt.get<NetworkHelper>().cookieJar.get(url.toHttpUrl()).firstOrNull { it.name == name }?.value
    }.getOrNull().orEmpty()

    override fun request(sourceId: Long, requestJson: String): String = try {
        val req = Json.parseToJsonElement(requestJson).jsonObject
        val source = RustSources.byId(sourceId)
        val url = req["url"]!!.jsonPrimitive.content
        val method = req["method"]?.jsonPrimitive?.contentOrNull?.uppercase()?.ifBlank { null } ?: "GET"

        val headers = (source?.headers?.newBuilder() ?: okhttp3.Headers.Builder()).apply {
            (req["headers"] as? JsonArray)?.forEach { pair ->
                val p = pair.jsonArray
                set(p[0].jsonPrimitive.content, p[1].jsonPrimitive.content)
            }
        }.build()

        val body = req["body"]?.jsonPrimitive?.contentOrNull
        val contentType = req["content_type"]?.jsonPrimitive?.contentOrNull?.toMediaTypeOrNull()
        val request = Request.Builder().url(url).headers(headers).method(
            method,
            when {
                method == "GET" || method == "HEAD" -> null
                else -> (body ?: "").toRequestBody(contentType)
            },
        ).build()

        var client = source?.client ?: Injekt.get<NetworkHelper>().client
        if ((req["follow_redirects"] as? JsonPrimitive)?.booleanOrNull == false) {
            client = client.newBuilder().followRedirects(false).followSslRedirects(false).build()
        }
        client.newCall(request).execute().use { response ->
            buildJsonObject {
                put("status", response.code)
                put("url", response.request.url.toString())
                put(
                    "headers",
                    buildJsonArray {
                        response.headers.forEach { (name, value) ->
                            add(buildJsonArray { add(JsonPrimitive(name)); add(JsonPrimitive(value)) })
                        }
                    },
                )
                if ((req["binary"] as? JsonPrimitive)?.booleanOrNull == true) {
                    val max = (req["max_bytes"] as? JsonPrimitive)?.longOrNull
                    val bytes = if (max != null) response.peekBody(max).bytes() else response.body.bytes()
                    put("body_b64", Base64.encodeToString(bytes, Base64.NO_WRAP))
                } else {
                    put("body", response.body.string())
                }
            }.toString()
        }
    } catch (e: Exception) {
        buildJsonObject { put("error", e.message ?: e.javaClass.simpleName) }.toString()
    }
}
// KMK <--

internal fun JsonObject.str(key: String): String? = (this[key] as? JsonPrimitive)?.contentOrNull
