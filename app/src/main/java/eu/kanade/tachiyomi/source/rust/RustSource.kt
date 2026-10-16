package eu.kanade.tachiyomi.source.rust

import eu.kanade.tachiyomi.source.model.FilterList
import eu.kanade.tachiyomi.source.model.MangasPage
import eu.kanade.tachiyomi.source.model.Page
import eu.kanade.tachiyomi.source.model.SChapter
import eu.kanade.tachiyomi.source.model.SManga
import eu.kanade.tachiyomi.source.model.SMangaUpdate
import eu.kanade.tachiyomi.source.online.HttpSource
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.floatOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put
import okhttp3.Headers
import uniffi.sk_api.RustSourceInfo
import uniffi.sk_api.callRustSource

// KMK -->
/**
 * A source written in Rust and compiled to WebAssembly (see `docs/redesign/RUST-SOURCES.md`).
 * Extends [HttpSource] so covers and chapter images go through the app's own network stack (cookies,
 * Cloudflare handling, referer) like any extension.
 */
class RustSource(private val info: RustSourceInfo) : HttpSource() {

    override val id: Long = info.id
    override val name: String = info.name
    override val lang: String = info.lang
    override val baseUrl: String = info.baseUrl
    override val supportsLatest: Boolean = info.supportsLatest

    // The default related-titles path builds OkHttp requests the Rust ABI does not have.
    override val supportsRelatedMangas: Boolean = false

    override fun headersBuilder(): Headers.Builder = super.headersBuilder().add("Referer", "$baseUrl/")

    override suspend fun getPopularManga(page: Int): MangasPage = mangasPage("popular") { put("page", page) }

    override suspend fun getLatestUpdates(page: Int): MangasPage = mangasPage("latest") { put("page", page) }

    override suspend fun getSearchManga(page: Int, query: String, filters: FilterList): MangasPage =
        mangasPage("search") {
            put("page", page)
            put("query", query)
        }

    override suspend fun getMangaUpdate(
        manga: SManga,
        chapters: List<SChapter>,
        fetchDetails: Boolean,
        fetchChapters: Boolean,
    ): SMangaUpdate {
        val details = if (fetchDetails) {
            call("details") { put("url", manga.url) }["manga"]!!.jsonObject.toManga()
        } else {
            manga
        }
        val chapterList = if (fetchChapters) {
            call("chapters") { put("url", manga.url) }["chapters"]!!.jsonArray.map { it.jsonObject.toChapter() }
        } else {
            chapters
        }
        return SMangaUpdate(details, chapterList)
    }

    override suspend fun getPageList(chapter: SChapter): List<Page> =
        call("pages") { put("url", chapter.url) }["pages"]!!.jsonArray.map {
            val p = it.jsonObject
            Page(
                index = p.int("index") ?: 0,
                url = p.string("url").orEmpty(),
                imageUrl = p.string("image_url"),
            )
        }

    private suspend fun mangasPage(op: String, params: kotlinx.serialization.json.JsonObjectBuilder.() -> Unit): MangasPage {
        val page = call(op, params)["page"]!!.jsonObject
        return MangasPage(
            mangas = page["mangas"]!!.jsonArray.map { it.jsonObject.toManga() },
            hasNextPage = (page["has_next"] as? JsonPrimitive)?.booleanOrNull ?: false,
        )
    }

    /** Runs one request in the WebAssembly module and returns the `ok` payload, or throws the source's error. */
    private suspend fun call(op: String, params: kotlinx.serialization.json.JsonObjectBuilder.() -> Unit): JsonObject =
        withContext(Dispatchers.IO) {
            val request = buildJsonObject {
                put("op", op)
                params()
            }.toString()
            val reply = Json.parseToJsonElement(callRustSource(id, request)).jsonObject
            reply["error"]?.let { throw RustSourceException((it as? JsonPrimitive)?.contentOrNull ?: it.toString()) }
            reply
        }

    private fun JsonObject.toManga(): SManga = SManga.create().also {
        it.url = string("url").orEmpty()
        it.title = string("title").orEmpty()
        it.thumbnail_url = string("thumbnail_url")
        it.author = string("author")
        it.artist = string("artist")
        it.description = string("description")
        it.genre = (this["genre"] as? JsonArray)?.mapNotNull { g -> (g as? JsonPrimitive)?.contentOrNull }
            ?.joinToString(", ")
            ?.takeIf { g -> g.isNotBlank() }
        it.status = int("status") ?: SManga.UNKNOWN
        it.initialized = this["description"] != null
    }

    private fun JsonObject.toChapter(): SChapter = SChapter.create().also {
        it.url = string("url").orEmpty()
        it.name = string("name").orEmpty()
        it.chapter_number = (this["chapter_number"] as? JsonPrimitive)?.floatOrNull ?: -1f
        it.scanlator = string("scanlator")
        it.date_upload = (this["date_upload"] as? JsonPrimitive)?.longOrNull ?: 0L
    }

    private fun JsonObject.string(key: String): String? = (this[key] as? JsonPrimitive)?.contentOrNull

    private fun JsonObject.int(key: String): Int? = (this[key] as? JsonPrimitive)?.intOrNull
}

class RustSourceException(message: String) : Exception(message)
// KMK <--
