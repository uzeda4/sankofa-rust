package eu.kanade.tachiyomi.rustcore

import android.content.Context
import logcat.LogPriority
import tachiyomi.core.common.util.system.logcat
import uniffi.sk_api.CoreInfo
import uniffi.sk_api.coreInfo

// KMK -->
/**
 * Entry point to the Rust core (`rust/crates/sk-api`). The native library is built per ABI by
 * `rust/tools/build-android.ps1`; on an ABI without it every call returns null instead of crashing.
 */
object RustCore {

    /** Reads the app's own database from Rust. Blocking: call from a background dispatcher. */
    fun info(context: Context): CoreInfo? = runCatching {
        coreInfo(context.getDatabasePath(DB_NAME).absolutePath)
    }.onFailure { logcat(LogPriority.WARN, it) { "Rust core unavailable" } }.getOrNull()

    private const val DB_NAME = "tachiyomi.db"
}
// KMK <--
