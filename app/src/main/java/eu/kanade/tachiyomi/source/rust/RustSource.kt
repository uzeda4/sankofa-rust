package eu.kanade.tachiyomi.source.rust

import android.net.Uri
import android.os.SystemClock
import eu.kanade.tachiyomi.network.interceptor.rateLimit
import eu.kanade.tachiyomi.network.interceptor.rateLimitHost
import eu.kanade.tachiyomi.source.ConfigurableSource
import eu.kanade.tachiyomi.source.PreferenceScreen
import eu.kanade.tachiyomi.source.model.FilterList
import eu.kanade.tachiyomi.source.model.MangasPage
import eu.kanade.tachiyomi.source.model.Page
import eu.kanade.tachiyomi.source.model.SChapter
import eu.kanade.tachiyomi.source.model.SManga
import eu.kanade.tachiyomi.source.model.SMangaUpdate
import eu.kanade.tachiyomi.source.online.HttpSource
import eu.kanade.tachiyomi.source.online.ResolvableSource
import eu.kanade.tachiyomi.source.online.UriType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonObjectBuilder
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.floatOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put
import eu.kanade.tachiyomi.network.NetworkHelper
import okhttp3.Cookie
import okhttp3.Headers
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.OkHttpClient
import uniffi.sk_api.RustSourceInfo
import uniffi.sk_api.callRustSource
import uniffi.sk_api.isUrlAllowed
import logcat.LogPriority
import tachiyomi.core.common.util.system.logcat
import uy.kohesive.injekt.Injekt
import uy.kohesive.injekt.api.get
import java.io.IOException
import kotlin.time.Duration.Companion.milliseconds

// KMK -->
/**
 * A source written in Rust and compiled to WebAssembly (see `docs/redesign/RUST-SOURCES.md`).
 * Extends [HttpSource] so covers and chapter images go through the app's own network stack (cookies,
 * Cloudflare handling, referer) like any extension, and the source's own requests use the same client.
 */
class RustSource(private val info: RustSourceInfo) : HttpSource(), ConfigurableSource, ResolvableSource {

    private val meta: JsonObject = runCatching { Json.parseToJsonElement(info.infoJson).jsonObject }.getOrDefault(JsonObject(emptyMap()))
    private val mirrors: List<String> = (meta["mirrors"] as? JsonArray).orEmpty().map { it.text() }

    /** Declared settings, with a Domain choice in front when the site has several addresses. */
    private val prefDefs: JsonArray = JsonArray(
        listOfNotNull(
            mirrors.takeIf { it.size > 1 }?.let {
                buildJsonObject {
                    put("type", "list")
                    put("key", "mirror")
                    put("title", "Domain")
                    put("default", info.baseUrl)
                    put("entries", buildJsonArray { it.forEach { m -> add(buildJsonArray { add(JsonPrimitive(m)); add(JsonPrimitive(m)) }) } })
                }
            },
        ) + (meta["prefs"] as? JsonArray).orEmpty(),
    )

    override val id: Long = info.id
    override val name: String = info.name
    override val lang: String = info.lang
    override val baseUrl: String
        get() = if (mirrors.size > 1) getSourcePreferences().getString("mirror", null)?.trimEnd('/') ?: info.baseUrl else info.baseUrl
    override val supportsLatest: Boolean = info.supportsLatest

    // The default related-titles path builds OkHttp requests the Rust ABI does not have.
    override val supportsRelatedMangas: Boolean = (meta["supports_related"] as? JsonPrimitive)?.booleanOrNull == true

    private val supportsFilterFetching = (meta["supports_filter_fetching"] as? JsonPrimitive)?.booleanOrNull == true

    override fun headersBuilder(): Headers.Builder = super.headersBuilder().add("Referer", "$baseUrl/").apply {
        (meta["headers"] as? JsonArray)?.forEach { h -> set(h.jsonArray[0].text(), h.jsonArray[1].text()) }
    }

    /** The app's client plus what the source declared: a rate limit, image edits, and the private-network guard. */
    override val client: OkHttpClient by lazy {
        applyCookies()
        super.client.newBuilder().apply {
            (meta["rate_limit"] as? JsonObject)?.let { r ->
                val permits = (r["permits"] as? JsonPrimitive)?.intOrNull ?: return@let
                val period = (r["period_ms"] as? JsonPrimitive)?.longOrNull ?: return@let
                val hosts = (r["host"] as? JsonPrimitive)?.contentOrNull.orEmpty().split(',').map { it.trim() }.filter { it.isNotEmpty() }
                if (hosts.isEmpty()) {
                    rateLimit(permits, period.milliseconds)
                } else {
                    hosts.forEach { rateLimitHost("https://$it", permits, period.milliseconds) }
                }
            }
            addInterceptor(RustImageInterceptor())
            // Every hop, redirects included: a public address must not send a source to a private one.
            addNetworkInterceptor { chain ->
                if (!isUrlAllowed(chain.request().url.toString())) throw IOException("blocked: ${chain.request().url.host} is a local or private address")
                chain.proceed(chain.request())
            }
        }.build()
    }

    /** Cookies the source asks for (an age check, a reading mode) go into the app's cookie jar. */
    private fun applyCookies() {
        val jar = Injekt.get<NetworkHelper>().cookieJar
        (meta["cookies"] as? JsonArray).orEmpty().forEach { c ->
            val o = c.jsonObject
            val host = o.text("host") ?: return@forEach
            val cookie = Cookie.Builder().name(o.text("name").orEmpty()).value(o.text("value").orEmpty()).domain(host).path("/").build()
            jar.saveFromResponse("https://$host/".toHttpUrl(), listOf(cookie))
        }
    }

    // ------------------------------------------------------------------ browsing

    override suspend fun getPopularManga(page: Int): MangasPage = mangasPage("popular") { put("page", page) }

    override suspend fun getLatestUpdates(page: Int): MangasPage = mangasPage("latest") { put("page", page) }

    override suspend fun getSearchManga(page: Int, query: String, filters: FilterList): MangasPage =
        mangasPage("search") {
            put("page", page)
            put("query", query)
            put("filters", RustFilters.state(filters))
        }

    override fun getFilterList(): FilterList {
        refreshFilterData()
        return runCatching {
            val reply = callBlocking("filters") { RustFilterData.get(id)?.let { put("data", it) } }
            RustFilters.build(reply["filters"]!!.jsonArray)
        }.getOrDefault(FilterList())
    }

    /** Genre lists and the like come from the site; fetched once a day in the background and cached. */
    private fun refreshFilterData() {
        if (!supportsFilterFetching || !RustFilterData.isStale(id)) return
        RustFilterData.fetchOnce(id) {
            val data = callBlocking("filter_data") {}["data"]
            if (data != null && data !is JsonNull) RustFilterData.put(id, data)
        }
    }

    override suspend fun getMangaUpdate(
        manga: SManga,
        chapters: List<SChapter>,
        fetchDetails: Boolean,
        fetchChapters: Boolean,
    ): SMangaUpdate {
        val update = call("update") {
            put("manga", manga.toJson())
            put("fetch_details", fetchDetails)
            put("fetch_chapters", fetchChapters)
        }["update"]!!.jsonObject
        val updatedManga = (update["manga"] as? JsonObject)?.toManga(manga) ?: manga
        val updatedChapters = (update["chapters"] as? JsonArray)?.map { it.jsonObject.toChapter() } ?: chapters
        return SMangaUpdate(updatedManga, updatedChapters)
    }

    override suspend fun getPageList(chapter: SChapter): List<Page> =
        call("pages") { put("chapter", chapter.toJson()) }["pages"]!!.jsonArray.mapIndexed { i, it ->
            val p = it.jsonObject
            val image = p.text("image_url")
            Page(
                index = (p["index"] as? JsonPrimitive)?.intOrNull ?: i,
                url = p.text("url").orEmpty(),
                imageUrl = image?.let { url -> RustImages.encode(url, p) },
            )
        }

    override suspend fun fetchRelatedMangaList(manga: SManga): List<SManga> =
        call("related") { put("manga", manga.toJson()) }["mangas"]!!.jsonArray.map { it.jsonObject.toManga(null) }

    // ------------------------------------------------------------------ opening links

    private val customUrls = (meta["custom_urls"] as? JsonPrimitive)?.booleanOrNull == true

    /** Some sites open a title at an address that is not the stored one (a random suffix, an id). */
    override fun getMangaUrl(manga: SManga): String = (if (customUrls) openUrl("manga_url") { put("manga", manga.toJson()) } else null) ?: super.getMangaUrl(manga)

    override fun getChapterUrl(chapter: SChapter): String = (if (customUrls) openUrl("chapter_url") { put("chapter", chapter.toJson()) } else null) ?: super.getChapterUrl(chapter)

    private fun openUrl(op: String, params: JsonObjectBuilder.() -> Unit): String? =
        runCatching { callBlocking(op, params)["url"]?.takeIf { it !is JsonNull }?.text() }.getOrNull()

    override fun getUriType(uri: String): UriType {
        val host = Uri.parse(uri).host ?: return UriType.Unknown
        return if (host == Uri.parse(baseUrl).host) UriType.Manga else UriType.Unknown
    }

    override suspend fun getManga(uri: String): SManga? =
        (call("by_url") { put("url", uri) }["manga"] as? JsonObject)?.toManga(null)

    override suspend fun getChapter(uri: String): SChapter? = null

    // ------------------------------------------------------------------ settings

    override fun setupPreferenceScreen(screen: PreferenceScreen) = RustPrefs.setup(screen, prefDefs)

    // ------------------------------------------------------------------ plumbing

    private suspend fun mangasPage(op: String, params: JsonObjectBuilder.() -> Unit): MangasPage {
        val page = call(op, params)["page"]!!.jsonObject
        return MangasPage(
            mangas = page["mangas"]!!.jsonArray.map { it.jsonObject.toManga(null) },
            hasNextPage = (page["has_next"] as? JsonPrimitive)?.booleanOrNull ?: false,
        )
    }

    /** Runs one request in the WebAssembly module and returns the answer, or throws the source's error. */
    private suspend fun call(op: String, params: JsonObjectBuilder.() -> Unit): JsonObject =
        withContext(Dispatchers.IO) { callBlocking(op, params) }

    private fun callBlocking(op: String, params: JsonObjectBuilder.() -> Unit): JsonObject {
        val request = buildJsonObject {
            put("op", op)
            params()
            put(
                "ctx",
                buildJsonObject {
                    put("prefs", RustPrefs.snapshot(getSourcePreferences(), prefDefs))
                    RustFilterData.get(id)?.let { put("filter_data", it) }
                },
            )
        }.toString()
        val started = SystemClock.elapsedRealtime()
        val answer = callRustSource(id, request)
        // Includes the time the source waited for the network; compare with OkHttp's own log to see its share.
        logcat(LogPriority.DEBUG) { "Rust source $name: $op in ${SystemClock.elapsedRealtime() - started} ms (${answer.length} bytes)" }
        val reply = Json.parseToJsonElement(answer).jsonObject
        reply["error"]?.let { throw RustSourceException((it as? JsonPrimitive)?.contentOrNull ?: it.toString()) }
        return reply
    }

    private fun SManga.toJson(): JsonObject = buildJsonObject {
        put("url", url)
        put("title", title)
        put("thumbnail_url", thumbnail_url)
        put("author", author)
        put("artist", artist)
        put("description", description)
        put("genre", buildJsonArray { getGenres()?.forEach { add(JsonPrimitive(it)) } })
        put("status", status)
        put("initialized", initialized)
        put("memo", memo)
    }

    private fun SChapter.toJson(): JsonObject = buildJsonObject {
        put("url", url)
        put("name", name)
        put("chapter_number", chapter_number)
        put("scanlator", scanlator)
        put("date_upload", date_upload)
        put("memo", memo)
    }

    /** Fills a new `SManga`, or copies `base` first so what the source leaves out (a cover) is kept. */
    private fun JsonObject.toManga(base: SManga?): SManga = (base?.copy() ?: SManga.create()).also {
        it.url = text("url") ?: base?.url.orEmpty()
        it.title = text("title") ?: base?.title.orEmpty()
        text("thumbnail_url")?.let { t -> it.thumbnail_url = t }
        if (base == null || this["author"] != null) it.author = text("author")
        if (base == null || this["artist"] != null) it.artist = text("artist")
        if (base == null || this["description"] != null) it.description = text("description")
        (this["genre"] as? JsonArray)?.mapNotNull { g -> (g as? JsonPrimitive)?.contentOrNull }
            ?.takeIf { g -> g.isNotEmpty() }
            ?.let { g -> it.genre = g.joinToString(", ") }
        it.status = (this["status"] as? JsonPrimitive)?.intOrNull ?: SManga.UNKNOWN
        it.initialized = (this["initialized"] as? JsonPrimitive)?.booleanOrNull ?: (this["description"] != null)
        (this["memo"] as? JsonObject)?.let { m -> it.memo = m }
    }

    private fun JsonObject.toChapter(): SChapter = SChapter.create().also {
        it.url = text("url").orEmpty()
        it.name = text("name").orEmpty()
        it.chapter_number = (this["chapter_number"] as? JsonPrimitive)?.floatOrNull ?: -1f
        it.scanlator = text("scanlator")
        it.date_upload = (this["date_upload"] as? JsonPrimitive)?.longOrNull ?: 0L
        (this["memo"] as? JsonObject)?.let { m -> it.memo = m }
    }

    private fun JsonObject.text(key: String): String? = (this[key] as? JsonPrimitive)?.contentOrNull

    private fun JsonElement.text(): String = (this as JsonPrimitive).content
}

class RustSourceException(message: String) : Exception(message)
// KMK <--
