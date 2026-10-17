package eu.kanade.presentation.more.settings.screen.browse

import android.content.Context
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import eu.kanade.presentation.components.AppBar
import eu.kanade.presentation.more.settings.widget.PreferenceGroupHeader
import eu.kanade.presentation.more.settings.widget.TextPreferenceWidget
import eu.kanade.presentation.util.Screen
import eu.kanade.tachiyomi.source.rust.RepoListing
import eu.kanade.tachiyomi.source.rust.RustRepo
import eu.kanade.tachiyomi.source.rust.RustSourceRepos
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
import uniffi.sk_api.RepoInfo
import uniffi.sk_api.SkException
import java.util.Locale

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
        var listings by remember { mutableStateOf<List<RepoListing>>(emptyList()) }
        var addUrl by remember { mutableStateOf<String?>(null) }
        var preview by remember { mutableStateOf<Pair<String, RepoInfo>?>(null) }
        var busyEntry by remember { mutableStateOf<String?>(null) }
        var languageFilter by remember { mutableStateOf(RustSourceRepos.languageFilter(context)) }
        val refresh: () -> Unit = {
            scope.launch { listings = withContext(Dispatchers.IO) { RustSourceRepos.refresh(context) } }
        }
        LaunchedEffect(Unit) { refresh() }

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
                item { PreferenceGroupHeader(title = stringResource(KMR.strings.rust_repos_header)) }
                item {
                    TextPreferenceWidget(
                        title = stringResource(KMR.strings.rust_repos_add),
                        subtitle = stringResource(KMR.strings.rust_repos_add_summary),
                        onPreferenceClick = { addUrl = "" },
                    )
                }
                listings.forEach { l ->
                    item(key = "repo-${l.repo.url}") {
                        TextPreferenceWidget(
                            title = l.repo.name,
                            subtitle = l.error?.let { stringResource(KMR.strings.rust_repos_error, it) }
                                ?: stringResource(
                                    KMR.strings.rust_repos_summary,
                                    l.repo.serial.toInt(),
                                    l.info?.sources?.size ?: 0,
                                    l.repo.fingerprint.take(FINGERPRINT_SHORT),
                                ),
                            widget = {
                                TextButton(
                                    onClick = {
                                        RustSourceRepos.remove(context, l.repo.url)
                                        refresh()
                                    },
                                ) { Text(stringResource(KMR.strings.rust_repos_remove)) }
                            },
                        )
                    }
                }
                val available = listings.flatMap { l -> l.info?.sources.orEmpty().map { l to it } }
                if (available.isNotEmpty()) {
                    item { PreferenceGroupHeader(title = stringResource(KMR.strings.rust_repos_available)) }
                }
                val languages = available.map { it.second.lang }.distinct().sortedBy { languageName(context, it).lowercase() }
                if (languages.size > 1) {
                    item(key = "avail-languages") {
                        LazyRow(
                            contentPadding = PaddingValues(horizontal = 16.dp),
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                        ) {
                            item {
                                FilterChip(
                                    selected = languageFilter.isEmpty(),
                                    onClick = {
                                        languageFilter = emptySet()
                                        RustSourceRepos.setLanguageFilter(context, languageFilter)
                                    },
                                    label = { Text(stringResource(KMR.strings.rust_repos_lang_all)) },
                                )
                            }
                            items(languages) { lang ->
                                FilterChip(
                                    selected = lang in languageFilter,
                                    onClick = {
                                        languageFilter = if (lang in languageFilter) languageFilter - lang else languageFilter + lang
                                        RustSourceRepos.setLanguageFilter(context, languageFilter)
                                    },
                                    label = { Text(languageName(context, lang)) },
                                )
                            }
                        }
                    }
                }
                available.filter { languageFilter.isEmpty() || it.second.lang in languageFilter }.forEach { (l, entry) ->
                    item(key = "avail-${l.repo.url}-${entry.name}-${entry.lang}") {
                        val installed = report?.sources?.firstOrNull { it.name == entry.name && it.lang == entry.lang }
                        val upToDate = installed != null && installed.version >= entry.version
                        TextPreferenceWidget(
                            icon = entry.iconUrl.ifEmpty { null },
                            title = "${entry.name} (${entry.lang.uppercase()})",
                            subtitle = if (installed != null) {
                                stringResource(KMR.strings.rust_repos_installed, installed.version.toInt(), entry.version.toInt())
                            } else {
                                "v${entry.version}"
                            },
                            widget = {
                                if (!upToDate) {
                                    TextButton(
                                        enabled = busyEntry == null,
                                        onClick = {
                                            busyEntry = entry.packageUrl
                                            scope.launch {
                                                runCatching { withContext(Dispatchers.IO) { RustSources.installFromRepo(context, entry) } }
                                                    .onSuccess { context.toast(context.stringResource(KMR.strings.rust_sources_installed_ok, entry.name)) }
                                                    .onFailure { context.toast(context.stringResource(KMR.strings.rust_sources_invalid, it.readable())) }
                                                busyEntry = null
                                            }
                                        },
                                    ) {
                                        Text(
                                            stringResource(
                                                if (installed == null) KMR.strings.rust_sources_install_now else KMR.strings.rust_repos_update,
                                            ),
                                        )
                                    }
                                }
                            },
                        )
                    }
                }
                item { PreferenceGroupHeader(title = stringResource(KMR.strings.rust_sources_installed_header)) }
                if (sources.isEmpty()) {
                    item {
                        TextPreferenceWidget(title = stringResource(KMR.strings.rust_sources_none))
                    }
                }
                sources.forEach { source ->
                    item(key = source.id) {
                        val info = report?.sources?.firstOrNull { it.id == source.id }
                        TextPreferenceWidget(
                            icon = available.firstOrNull { (_, e) -> e.name == source.name && e.iconUrl.isNotEmpty() }?.second?.iconUrl,
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

        addUrl?.let { url ->
            AlertDialog(
                onDismissRequest = { addUrl = null },
                title = { Text(stringResource(KMR.strings.rust_repos_url_title)) },
                text = {
                    OutlinedTextField(
                        value = url,
                        onValueChange = { addUrl = it },
                        singleLine = true,
                        // An address: no auto-capitalization or autocorrect.
                        keyboardOptions = KeyboardOptions(
                            capitalization = KeyboardCapitalization.None,
                            autoCorrectEnabled = false,
                            keyboardType = KeyboardType.Uri,
                        ),
                        placeholder = { Text("https://example.org/index.skrepo") },
                    )
                },
                confirmButton = {
                    TextButton(
                        enabled = url.isNotBlank(),
                        onClick = {
                            addUrl = null
                            scope.launch {
                                runCatching { withContext(Dispatchers.IO) { RustSourceRepos.preview(url.trim()) } }
                                    .onSuccess { preview = url.trim() to it }
                                    .onFailure { context.toast(context.stringResource(KMR.strings.rust_sources_invalid, it.readable())) }
                            }
                        },
                    ) { Text(stringResource(KMR.strings.rust_repos_continue)) }
                },
                dismissButton = {
                    TextButton(onClick = { addUrl = null }) { Text(stringResource(MR.strings.action_cancel)) }
                },
            )
        }

        preview?.let { (url, info) ->
            AlertDialog(
                onDismissRequest = { preview = null },
                title = { Text(stringResource(KMR.strings.rust_repos_confirm_title, info.name)) },
                text = {
                    Text(stringResource(KMR.strings.rust_repos_confirm_body, info.fingerprint, info.sources.size))
                },
                confirmButton = {
                    TextButton(
                        onClick = {
                            preview = null
                            RustSourceTrust.trust(context, info.fingerprint)
                            RustSourceRepos.add(context, RustRepo(url, info.name, info.fingerprint, info.serial.toLong()))
                            refresh()
                        },
                    ) { Text(stringResource(KMR.strings.rust_repos_confirm)) }
                },
                dismissButton = {
                    TextButton(onClick = { preview = null }) { Text(stringResource(MR.strings.action_cancel)) }
                },
            )
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

    /** `en` as "English", `es-419` as "Spanish (Latin America)", the special codes by name. */
    private fun languageName(context: Context, lang: String): String = when (lang) {
        "all" -> context.stringResource(KMR.strings.rust_repos_lang_multi)
        "other" -> context.stringResource(KMR.strings.rust_repos_lang_other)
        else -> Locale.forLanguageTag(lang).getDisplayName(Locale.getDefault()).replaceFirstChar { it.uppercase() }.ifBlank { lang }
    }

    private companion object {
        const val FINGERPRINT_SHORT = 8
    }
}
// KMK <--
