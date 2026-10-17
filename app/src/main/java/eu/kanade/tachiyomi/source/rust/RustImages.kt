package eu.kanade.tachiyomi.source.rust

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Rect
import android.os.Build
import android.text.Layout
import android.text.StaticLayout
import android.text.TextPaint
import android.util.Base64
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.HttpUrl.Companion.toHttpUrlOrNull
import okhttp3.Interceptor
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.Protocol
import okhttp3.Request
import okhttp3.Response
import okhttp3.ResponseBody.Companion.toResponseBody
import java.io.ByteArrayOutputStream
import java.io.IOException

// KMK -->
/**
 * What a page of a Rust source asks of its image, carried in the URL fragment (which never goes on the wire):
 * extra headers, mirrors to try when the image fails, and edits to apply (merge strips, undo a shuffle, draw text).
 * The source describes the work; the app does it here, where a `Bitmap` is fast.
 */
object RustImages {

    private const val PREFIX = "sk="

    fun encode(imageUrl: String, page: JsonObject): String {
        val spec = buildJsonObject {
            page["headers"]?.let { if ((it as? JsonArray)?.isNotEmpty() == true) put("h", it) }
            page["mirrors"]?.let { if ((it as? JsonArray)?.isNotEmpty() == true) put("alt", it) }
            page["ops"]?.let { if ((it as? JsonArray)?.isNotEmpty() == true) put("ops", it) }
        }
        if (spec.isEmpty()) return imageUrl
        val payload = Base64.encodeToString(spec.toString().toByteArray(), Base64.URL_SAFE or Base64.NO_WRAP or Base64.NO_PADDING)
        return "${imageUrl.substringBefore('#')}#$PREFIX$payload"
    }

    fun decode(fragment: String?): JsonObject? {
        if (fragment == null || !fragment.startsWith(PREFIX)) return null
        return runCatching {
            Json.parseToJsonElement(String(Base64.decode(fragment.removePrefix(PREFIX), Base64.URL_SAFE))).jsonObject
        }.getOrNull()
    }
}

class RustImageInterceptor : Interceptor {

    override fun intercept(chain: Interceptor.Chain): Response {
        val request = chain.request()
        val spec = RustImages.decode(request.url.fragment) ?: return chain.proceed(request)

        val headers = request.headers.newBuilder().apply {
            (spec["h"] as? JsonArray)?.forEach { p -> set(p.jsonArray[0].str(), p.jsonArray[1].str()) }
        }.build()
        val base = request.newBuilder().headers(headers).build()
        val alts = (spec["alt"] as? JsonArray)?.map { it.str() }.orEmpty()
        val ops = (spec["ops"] as? JsonArray)?.map { it.jsonObject }.orEmpty()

        if (ops.isEmpty()) return fetch(chain, base, request.url.toString().substringBefore('#'), alts)

        var bitmap: Bitmap? = null
        try {
            for (op in ops) {
                bitmap = when (op.str("op")) {
                    "merge" -> merge(chain, base, op, alts)
                    "unscramble" -> unscramble(bitmap ?: decode(chain, base, request.url.toString().substringBefore('#'), alts), op)
                    "shuffle" -> shuffle(bitmap ?: decode(chain, base, request.url.toString().substringBefore('#'), alts), op)
                    "text" -> text(op)
                    else -> bitmap
                }
            }
            val out = bitmap ?: throw IOException("the source asked for an image edit the app does not know")
            return Response.Builder()
                .request(request)
                .protocol(Protocol.HTTP_1_1)
                .code(200)
                .message("OK")
                .body(encodeWebp(out).toResponseBody("image/webp".toMediaType()))
                .build()
        } finally {
            bitmap?.recycle()
        }
    }

    /** Fetches `url`, then the same path on each mirror until one answers with success. */
    private fun fetch(chain: Interceptor.Chain, base: Request, url: String, mirrors: List<String>): Response {
        var failure: IOException? = null
        val original = url.toHttpUrl()
        val candidates = listOf(url) + mirrors.mapNotNull { m ->
            m.toHttpUrlOrNull()?.let { origin ->
                original.newBuilder().scheme(origin.scheme).host(origin.host).port(origin.port).build().toString()
            }
        }
        for (candidate in candidates) {
            val response = try {
                chain.proceed(base.newBuilder().url(candidate).build())
            } catch (e: IOException) {
                failure = e
                continue
            }
            if (response.isSuccessful) return response
            response.close()
            failure = IOException("HTTP ${response.code} for $candidate")
        }
        throw failure ?: IOException("no image address")
    }

    private fun decode(chain: Interceptor.Chain, base: Request, url: String, alts: List<String>): Bitmap =
        fetch(chain, base, url, alts).use { response ->
            BitmapFactory.decodeStream(response.body.byteStream()) ?: throw IOException("cannot decode $url")
        }

    private fun merge(chain: Interceptor.Chain, base: Request, op: JsonObject, alts: List<String>): Bitmap {
        val urls = (op["urls"] as JsonArray).map { it.str() }
        val parts = ArrayList<Bitmap>(urls.size)
        try {
            urls.forEach { parts += decode(chain, base, it, alts) }
            // Sizes come from the pictures themselves rather than from the source's guess.
            val width = parts.maxOf { it.width }
            val result = Bitmap.createBitmap(width, parts.sumOf { it.height }, Bitmap.Config.ARGB_8888)
            val canvas = Canvas(result)
            var y = 0f
            parts.forEach {
                canvas.drawBitmap(it, 0f, y, null)
                y += it.height
            }
            return result
        } finally {
            parts.forEach { it.recycle() }
        }
    }

    private fun unscramble(source: Bitmap, op: JsonObject): Bitmap {
        val width = op["width"].int() ?: source.width
        val height = op["height"].int() ?: source.height
        val result = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(result)
        (op["tiles"] as JsonArray).forEach { t ->
            val tile = t.jsonObject
            val (sx, sy, dx, dy) = listOf("sx", "sy", "dx", "dy").map { tile[it].int() ?: 0 }
            val w = tile["w"].int() ?: 0
            val h = tile["h"].int() ?: 0
            canvas.drawBitmap(source, Rect(sx, sy, sx + w, sy + h), Rect(dx, dy, dx + w, dy + h), null)
        }
        source.recycle()
        return result
    }

    /** Tiles cut from a regular grid and shuffled: put each back where `order` says. */
    private fun shuffle(source: Bitmap, op: JsonObject): Bitmap {
        val cols = op["cols"].int() ?: throw IOException("shuffle without columns")
        val rows = op["rows"].int() ?: throw IOException("shuffle without rows")
        val order = (op["order"] as JsonArray).map { it.int() ?: 0 }
        val tileW = source.width / cols
        val tileH = source.height / rows
        val result = Bitmap.createBitmap(tileW * cols, tileH * rows, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(result)
        order.forEachIndexed { w, j ->
            val src = Rect((w % cols) * tileW, (w / cols) * tileH, (w % cols + 1) * tileW, (w / cols + 1) * tileH)
            val dst = Rect((j % cols) * tileW, (j / cols) * tileH, (j % cols + 1) * tileW, (j / cols + 1) * tileH)
            canvas.drawBitmap(source, src, dst, null)
        }
        source.recycle()
        return result
    }

    /** Text on a plain page (an author's note), 1080 px wide so it reads like the pages around it. */
    private fun text(op: JsonObject): Bitmap {
        val width = 1080
        val pad = 56
        val title = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
            color = Color.BLACK
            textSize = 56f
            isFakeBoldText = true
        }
        val body = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
            color = Color.BLACK
            textSize = 44f
        }
        val inner = width - pad * 2
        fun layout(text: String, paint: TextPaint) =
            StaticLayout.Builder.obtain(text, 0, text.length, paint, inner).setLineSpacing(8f, 1f).setAlignment(Layout.Alignment.ALIGN_NORMAL).build()
        val head = layout(op.str("title").orEmpty(), title)
        val rest = layout(op.str("body").orEmpty(), body)
        val result = Bitmap.createBitmap(width, pad * 3 + head.height + rest.height, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(result)
        canvas.drawColor(Color.WHITE)
        canvas.translate(pad.toFloat(), pad.toFloat())
        head.draw(canvas)
        canvas.translate(0f, (head.height + pad).toFloat())
        rest.draw(canvas)
        return result
    }

    private fun encodeWebp(bitmap: Bitmap): ByteArray = ByteArrayOutputStream().also {
        @Suppress("DEPRECATION")
        bitmap.compress(
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) Bitmap.CompressFormat.WEBP_LOSSLESS else Bitmap.CompressFormat.WEBP,
            100,
            it,
        )
    }.toByteArray()

    private fun kotlinx.serialization.json.JsonElement?.int(): Int? = (this as? JsonPrimitive)?.intOrNull
    private fun kotlinx.serialization.json.JsonElement.str(): String = (this as JsonPrimitive).content
}
// KMK <--
