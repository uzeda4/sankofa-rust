package eu.kanade.tachiyomi.source.rust

import android.content.Context
import logcat.LogPriority
import tachiyomi.core.common.util.system.logcat
import uniffi.sk_api.loadRustSources
import java.io.File

// KMK -->
/**
 * Sources written in Rust live as `.wasm` files in `files/rust-sources`. Loading is cheap (the modules are
 * only validated and instantiated); on an ABI without the native library this returns no sources.
 */
object RustSources {

    fun directory(context: Context): File = File(context.filesDir, "rust-sources").apply { mkdirs() }

    fun load(context: Context): List<RustSource> = runCatching {
        loadRustSources(directory(context).absolutePath).map(::RustSource)
    }.onFailure { logcat(LogPriority.WARN, it) { "Rust sources unavailable" } }.getOrDefault(emptyList())
}
// KMK <--
