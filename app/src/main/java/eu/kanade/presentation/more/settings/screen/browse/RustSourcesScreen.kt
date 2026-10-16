package eu.kanade.presentation.more.settings.screen.browse

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import eu.kanade.presentation.components.AppBar
import eu.kanade.presentation.more.settings.widget.TextPreferenceWidget
import eu.kanade.presentation.util.Screen
import eu.kanade.tachiyomi.source.rust.RustSourceTrust
import eu.kanade.tachiyomi.source.rust.RustSources
import eu.kanade.tachiyomi.util.system.toast
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import tachiyomi.core.common.i18n.stringResource
import tachiyomi.i18n.MR
import tachiyomi.i18n.kmk.KMR
import tachiyomi.presentation.core.components.ScrollbarLazyColumn
import tachiyomi.presentation.core.components.material.Scaffold
import tachiyomi.presentation.core.i18n.stringResource
import uniffi.sk_api.PackageInfo
import uniffi.sk_api.Rejected
import uniffi.sk_api.SkException

// KMK -->
/** Installs, trusts and removes sources written in Rust. See `docs/redesign/RUST-SOURCES.md`. */
class RustSourcesScreen : Screen() {

    /** A package the user picked and whose signature is valid; waiting for the trust decision. */
    private class Pending(val bytes: ByteArray, val info: PackageInfo)

    @Composable
    override fun Content() {
        val context = LocalContext.current
        val navigator = LocalNavigator.currentOrThrow
        val scope = rememberCoroutineScope()
        val sources by RustSources.sources.collectAsState()
        val report by RustSources.report.collectAsState()
        var pending by remember { mutableStateOf<Pending?>(null) }
        var toTrust by remember { mutableStateOf<Rejected?>(null) }

        val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
            if (uri == null) return@rememberLauncherForActivityResult
            scope.launch {
                runCatching {
                    withContext(Dispatchers.IO) {
                        val bytes = context.contentResolver.openInputStream(uri)!!.use { it.readBytes() }
                        Pending(bytes, RustSources.inspect(bytes))
                    }
                }.onSuccess { pending = it }
                    .onFailure { context.toast(context.stringResource(KMR.strings.rust_sources_invalid, it.readable())) }
            }
        }

        Scaffold(
            topBar = { scrollBehavior ->
                AppBar(
                    title = stringResource(KMR.strings.rust_sources_title),
                    navigateUp = navigator::pop,
                    scrollBehavior = scrollBehavior,
                )
            },
        ) { contentPadding ->
            ScrollbarLazyColumn(contentPadding = contentPadding) {
                item {
                    TextPreferenceWidget(
                        title = stringResource(KMR.strings.rust_sources_install),
                        subtitle = stringResource(KMR.strings.rust_sources_install_summary),
                        onPreferenceClick = { picker.launch(arrayOf("*/*")) },
                    )
                }
                if (sources.isEmpty()) {
                    item {
                        TextPreferenceWidget(title = stringResource(KMR.strings.rust_sources_none))
                    }
                }
                sources.forEach { source ->
                    item(key = source.id) {
                        val info = report?.sources?.firstOrNull { it.id == source.id }
                        TextPreferenceWidget(
                            title = "${source.name} (${source.lang.uppercase()})",
                            subtitle = if (info == null || info.fingerprint.isEmpty()) {
                                stringResource(KMR.strings.rust_sources_unsigned)
                            } else {
                                "v${info.version} · ${info.publisher} · ${info.fingerprint.take(FINGERPRINT_SHORT)}"
                            },
                            widget = {
                                if (info != null) {
                                    TextButton(
                                        onClick = {
                                            scope.launch {
                                                withContext(Dispatchers.IO) { RustSources.uninstall(context, info.fileName) }
                                            }
                                        },
                                    ) { Text(stringResource(KMR.strings.rust_sources_uninstall)) }
                                }
                            },
                        )
                    }
                }
                report?.rejected?.forEach { rejected ->
                    item(key = "rejected-${rejected.fileName}") {
                        TextPreferenceWidget(
                            title = rejected.fileName,
                            subtitle = if (rejected.fingerprint.isNotEmpty()) {
                                stringResource(KMR.strings.rust_sources_rejected_untrusted, rejected.fingerprint)
                            } else {
                                stringResource(KMR.strings.rust_sources_rejected, rejected.reason)
                            },
                            widget = {
                                if (rejected.fingerprint.isNotEmpty()) {
                                    TextButton(onClick = { toTrust = rejected }) {
                                        Text(stringResource(KMR.strings.rust_sources_trust))
                                    }
                                }
                            },
                        )
                    }
                }
            }
        }

        toTrust?.let { r ->
            AlertDialog(
                onDismissRequest = { toTrust = null },
                title = { Text(stringResource(KMR.strings.rust_sources_trust_dialog_title)) },
                text = { Text(stringResource(KMR.strings.rust_sources_trust_dialog_body, r.fingerprint)) },
                confirmButton = {
                    TextButton(
                        onClick = {
                            toTrust = null
                            RustSourceTrust.trust(context, r.fingerprint)
                            scope.launch { withContext(Dispatchers.IO) { RustSources.reload(context) } }
                        },
                    ) { Text(stringResource(KMR.strings.rust_sources_trust)) }
                },
                dismissButton = {
                    TextButton(onClick = { toTrust = null }) { Text(stringResource(MR.strings.action_cancel)) }
                },
            )
        }

        pending?.let { p ->
            val alreadyTrusted = p.info.fingerprint in RustSourceTrust.fingerprints(context)
            AlertDialog(
                onDismissRequest = { pending = null },
                title = { Text(stringResource(KMR.strings.rust_sources_trust_title, p.info.name)) },
                text = {
                    Text(
                        stringResource(KMR.strings.rust_sources_trust_body, p.info.version.toInt(), p.info.publisher, p.info.fingerprint),
                        style = MaterialTheme.typography.bodyMedium,
                    )
                },
                confirmButton = {
                    TextButton(
                        onClick = {
                            pending = null
                            scope.launch {
                                runCatching {
                                    withContext(Dispatchers.IO) {
                                        if (!alreadyTrusted) RustSourceTrust.trust(context, p.info.fingerprint)
                                        RustSources.install(context, p.bytes)
                                    }
                                }.onSuccess { context.toast(context.stringResource(KMR.strings.rust_sources_installed_ok, p.info.name)) }
                                    .onFailure { context.toast(context.stringResource(KMR.strings.rust_sources_invalid, it.readable())) }
                            }
                        },
                    ) {
                        Text(
                            stringResource(
                                if (alreadyTrusted) KMR.strings.rust_sources_install_now else KMR.strings.rust_sources_trust_and_install,
                            ),
                        )
                    }
                },
                dismissButton = {
                    TextButton(onClick = { pending = null }) { Text(stringResource(MR.strings.action_cancel)) }
                },
            )
        }
    }

    /** UniFFI prints errors as "reason=..."; show just the reason. */
    private fun Throwable.readable(): String = (this as? SkException.Package)?.reason ?: message.orEmpty()

    private companion object {
        const val FINGERPRINT_SHORT = 8
    }
}
// KMK <--
