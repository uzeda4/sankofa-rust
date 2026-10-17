package eu.kanade.tachiyomi.source.rust

import android.content.Context
import androidx.core.content.edit
import eu.kanade.tachiyomi.BuildConfig
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put
import logcat.LogPriority
import tachiyomi.core.common.util.system.logcat
import uniffi.sk_api.LoadReport
import uniffi.sk_api.PackageInfo
import uniffi.sk_api.RepoEntryInfo
import uniffi.sk_api.RepoInfo
import uniffi.sk_api.fetchRepoIndex
import uniffi.sk_api.inspectSourcePackage
import uniffi.sk_api.installSourcePackage
import uniffi.sk_api.loadRustSources
import uniffi.sk_api.setHttpBridge
import uniffi.sk_api.uninstallSource
import uy.kohesive.injekt.Injekt
import uy.kohesive.injekt.api.get
import java.io.File
import java.util.concurrent.ConcurrentHashMap
import uniffi.sk_api.installFromRepo as installRepoPackage

// KMK -->
/**
 * Sources written in Rust (WebAssembly) live in `files/rust-sources`. Installed ones are signed `.sksource`
 * packages and run only for publishers the user trusts; raw `.wasm` files load in debug builds only.
 * On an ABI without the native library everything here returns empty instead of crashing.
 */
object RustSources {

    private val _sources = MutableStateFlow<List<RustSource>>(emptyList())
    val sources: StateFlow<List<RustSource>> = _sources.asStateFlow()

    private val _report = MutableStateFlow<LoadReport?>(null)

    /** What the last load accepted and rejected (for example a package from a publisher not yet trusted). */
    val report: StateFlow<LoadReport?> = _report.asStateFlow()

    fun directory(context: Context): File = File(context.filesDir, "rust-sources").apply { mkdirs() }

    fun byId(id: Long): RustSource? = _sources.value.firstOrNull { it.id == id }

    /** Sources reach the network through the app's client once this is set (before the first source is loaded). */
    private val bridgeInstalled by lazy { setHttpBridge(RustHttpBridge) }

    /** Re-reads the folder; call after installing, uninstalling or changing trust. Blocking. */
    @Synchronized
    fun reload(context: Context) {
        runCatching {
            bridgeInstalled
            loadRustSources(directory(context).absolutePath, RustSourceTrust.fingerprints(context).toList(), BuildConfig.DEBUG)
        }.onSuccess { report ->
            _report.value = report
            _sources.value = report.sources.map(::RustSource)
            logcat(LogPriority.INFO) { "Rust sources loaded: ${report.sources.joinToString { "${it.name} (${it.lang}) #${it.id}" }}; rejected: ${report.rejected.joinToString { "${it.fileName}: ${it.reason}" }}" }
        }.onFailure { logcat(LogPriority.WARN, it) { "Rust sources unavailable" } }
    }

    /**
     * Walks a source through what a reader does (browse, search, open a title, open a chapter, fetch a picture) and
     * logs the sizes, so a debug start on a device shows which sources work. Each step is independent.
     */
    suspend fun smokeTest(source: RustSource) {
        suspend fun <T> step(name: String, block: suspend () -> T): T? = runCatching { block() }
            .onFailure { logcat(LogPriority.WARN, it) { "Rust smoke ${source.name}: $name FAILED: ${it.message}" } }
            .getOrNull()

        val popular = step("popular") { source.getPopularManga(1) } ?: return
        logcat(LogPriority.INFO) { "Rust smoke ${source.name}: popular ${popular.mangas.size}, first '${popular.mangas.firstOrNull()?.title}'" }
        if (source.supportsLatest) {
            step("latest") { source.getLatestUpdates(1) }?.let { logcat(LogPriority.INFO) { "Rust smoke ${source.name}: latest ${it.mangas.size}" } }
        }
        popular.mangas.firstOrNull()?.title?.split(' ')?.firstOrNull()?.let { word ->
            step("search") { source.getSearchManga(1, word, source.getFilterList()) }
                ?.let { logcat(LogPriority.INFO) { "Rust smoke ${source.name}: search '$word' ${it.mangas.size}" } }
        }
        val manga = popular.mangas.firstOrNull() ?: return
        val update = step("title") { source.getMangaUpdate(manga, emptyList(), true, true) } ?: return
        logcat(LogPriority.INFO) {
            "Rust smoke ${source.name}: title '${update.manga.title}' status ${update.manga.status}, " +
                "${update.manga.genre?.split(", ")?.size ?: 0} genres, desc ${update.manga.description?.length ?: 0} chars, ${update.chapters.size} chapters"
        }
        val chapter = update.chapters.firstOrNull() ?: return
        val pages = step("pages") { source.getPageList(chapter) } ?: return
        logcat(LogPriority.INFO) { "Rust smoke ${source.name}: chapter '${chapter.name}' ${pages.size} pages" }
        val first = pages.firstOrNull() ?: return
        step("image") { source.getImage(first).use { r -> "${r.code} ${r.header("Content-Type")} ${r.body.bytes().size} bytes" } }
            ?.let { logcat(LogPriority.INFO) { "Rust smoke ${source.name}: first image $it" } }
    }

    /** Checks a package file's integrity and signature; trusts nothing. Throws on a bad package. */
    fun inspect(bytes: ByteArray): PackageInfo = inspectSourcePackage(bytes)

    /** Installs a package whose publisher is already trusted, then reloads. Throws with a readable reason. */
    fun install(context: Context, bytes: ByteArray) {
        installSourcePackage(bytes, directory(context).absolutePath, RustSourceTrust.fingerprints(context).toList())
        reload(context)
    }

    /** Downloads a package listed in a repository and installs it (signature and trusted publisher required). */
    fun installFromRepo(context: Context, entry: RepoEntryInfo) {
        installRepoPackage(
            entry.packageUrl,
            entry.sha256,
            directory(context).absolutePath,
            RustSourceTrust.fingerprints(context).toList(),
        )
        reload(context)
    }

    fun uninstall(context: Context, fileName: String) {
        uninstallSource(directory(context).absolutePath, fileName)
        reload(context)
    }
}

/**
 * Data that a source's filters need from its site (genre lists), cached for a day so opening the filter sheet
 * does not wait for the network.
 */
internal object RustFilterData {

    private const val PREFS = "rust_filter_data"
    private const val MAX_AGE_MS = 24L * 60 * 60 * 1000
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val running = ConcurrentHashMap.newKeySet<Long>()

    private fun prefs() = Injekt.get<android.app.Application>().getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    private fun entry(id: Long): JsonObject? = prefs().getString(id.toString(), null)?.let {
        runCatching { Json.parseToJsonElement(it).jsonObject }.getOrNull()
    }

    fun get(id: Long): JsonElement? = entry(id)?.get("data")

    fun isStale(id: Long): Boolean {
        val at = (entry(id)?.get("at") as? kotlinx.serialization.json.JsonPrimitive)?.content?.toLongOrNull() ?: return true
        return System.currentTimeMillis() - at > MAX_AGE_MS
    }

    fun put(id: Long, data: JsonElement) {
        val json = kotlinx.serialization.json.buildJsonObject {
            put("at", System.currentTimeMillis())
            put("data", data)
        }
        prefs().edit { putString(id.toString(), json.toString()) }
    }

    /** Runs `fetch` in the background unless one is already running for this source. */
    fun fetchOnce(id: Long, fetch: () -> Unit) {
        if (!running.add(id)) return
        scope.launch {
            try {
                fetch()
            } catch (e: Exception) {
                logcat(LogPriority.WARN, e) { "Filter data of source $id" }
            } finally {
                running.remove(id)
            }
        }
    }
}

/** Fingerprints of the publishers the user chose to trust. */
object RustSourceTrust {

    private const val PREFS = "rust_source_trust"
    private const val KEY = "fingerprints"

    fun fingerprints(context: Context): Set<String> =
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getStringSet(KEY, emptySet()).orEmpty()

    fun trust(context: Context, fingerprint: String) = update(context) { it + fingerprint }

    fun revoke(context: Context, fingerprint: String) = update(context) { it - fingerprint }

    private fun update(context: Context, change: (Set<String>) -> Set<String>) {
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        prefs.edit { putStringSet(KEY, change(prefs.getStringSet(KEY, emptySet()).orEmpty()).toSet()) }
    }
}

/** A repository the user added: where it is, who it is, and the newest index serial seen (anti-rollback). */
@Serializable
data class RustRepo(val url: String, val name: String, val fingerprint: String, val serial: Long = 0)

/** What a refresh found: sources per repository, and why a repository could not be read. */
class RepoListing(val repo: RustRepo, val info: RepoInfo?, val error: String?)

object RustSourceRepos {

    private const val PREFS = "rust_source_repos"
    private const val KEY = "repos"

    fun list(context: Context): List<RustRepo> = runCatching {
        Json.decodeFromString<List<RustRepo>>(
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(KEY, null) ?: "[]",
        )
    }.getOrDefault(emptyList())

    fun add(context: Context, repo: RustRepo) = save(context, list(context).filterNot { it.url == repo.url } + repo)

    fun remove(context: Context, url: String) = save(context, list(context).filterNot { it.url == url })

    private fun save(context: Context, repos: List<RustRepo>) {
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit { putString(KEY, Json.encodeToString(repos)) }
    }

    /** Fetches a repository for the first time so the user can confirm who it is. Throws with a readable reason. */
    fun preview(url: String): RepoInfo = fetchRepoIndex(url, "", 0uL)

    /** Re-reads every repository, refusing a changed key or an older index; remembers the newest serial. Blocking. */
    fun refresh(context: Context): List<RepoListing> = list(context).map { repo ->
        runCatching { fetchRepoIndex(repo.url, repo.fingerprint, repo.serial.toULong()) }
            .onSuccess { info -> if (info.serial.toLong() > repo.serial) add(context, repo.copy(serial = info.serial.toLong())) }
            .fold(
                onSuccess = { RepoListing(repo, it, null) },
                onFailure = { RepoListing(repo, null, (it as? uniffi.sk_api.SkException.Package)?.reason ?: it.message) },
            )
    }
}
// KMK <--
