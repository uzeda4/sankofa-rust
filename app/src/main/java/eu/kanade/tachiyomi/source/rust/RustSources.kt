package eu.kanade.tachiyomi.source.rust

import android.content.Context
import androidx.core.content.edit
import eu.kanade.tachiyomi.BuildConfig
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import logcat.LogPriority
import tachiyomi.core.common.util.system.logcat
import uniffi.sk_api.LoadReport
import uniffi.sk_api.PackageInfo
import uniffi.sk_api.inspectSourcePackage
import uniffi.sk_api.installSourcePackage
import uniffi.sk_api.loadRustSources
import uniffi.sk_api.uninstallSource
import java.io.File

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

    /** Re-reads the folder; call after installing, uninstalling or changing trust. Blocking. */
    @Synchronized
    fun reload(context: Context) {
        runCatching {
            loadRustSources(directory(context).absolutePath, RustSourceTrust.fingerprints(context).toList(), BuildConfig.DEBUG)
        }.onSuccess { report ->
            _report.value = report
            _sources.value = report.sources.map(::RustSource)
        }.onFailure { logcat(LogPriority.WARN, it) { "Rust sources unavailable" } }
    }

    /** Checks a package file's integrity and signature; trusts nothing. Throws on a bad package. */
    fun inspect(bytes: ByteArray): PackageInfo = inspectSourcePackage(bytes)

    /** Installs a package whose publisher is already trusted, then reloads. Throws with a readable reason. */
    fun install(context: Context, bytes: ByteArray) {
        installSourcePackage(bytes, directory(context).absolutePath, RustSourceTrust.fingerprints(context).toList())
        reload(context)
    }

    fun uninstall(context: Context, fileName: String) {
        uninstallSource(directory(context).absolutePath, fileName)
        reload(context)
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
// KMK <--
