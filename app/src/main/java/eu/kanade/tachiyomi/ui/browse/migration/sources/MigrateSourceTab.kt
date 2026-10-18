package eu.kanade.tachiyomi.ui.browse.migration.sources

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.HelpOutline
import androidx.compose.material.icons.outlined.SwapHoriz
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import cafe.adriel.voyager.core.model.rememberScreenModel
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import eu.kanade.presentation.browse.MigrateSourceScreen
import eu.kanade.presentation.components.AppBar
import eu.kanade.presentation.components.TabContent
import eu.kanade.tachiyomi.ui.browse.migration.manga.MigrateMangaScreen
import eu.kanade.tachiyomi.util.system.toast
import kotlinx.collections.immutable.persistentListOf
import tachiyomi.i18n.MR
import tachiyomi.i18n.kmk.KMR
import tachiyomi.core.common.i18n.stringResource
import tachiyomi.presentation.core.i18n.stringResource

@Composable
fun Screen.migrateSourceTab(): TabContent {
    val uriHandler = LocalUriHandler.current
    val navigator = LocalNavigator.currentOrThrow
    val screenModel = rememberScreenModel { MigrateSourceScreenModel() }
    val state by screenModel.state.collectAsState()
    // KMK -->
    val context = LocalContext.current
    LaunchedEffect(state.showRustMove) { if (state.showRustMove) screenModel.loadMoveTitles() }
    // KMK <--

    return TabContent(
        titleRes = MR.strings.label_migration,
        actions = persistentListOf(
            // KMK -->
            // titles of removed extensions that a Rust source of the same name can take over, with no search
            *listOfNotNull(
                AppBar.Action(
                    title = stringResource(KMR.strings.rust_move_title),
                    icon = Icons.Outlined.SwapHoriz,
                    onClick = { screenModel.showRustMove(true) },
                ).takeIf { state.rustMoves.isNotEmpty() },
            ).toTypedArray(),
            // KMK <--
            AppBar.Action(
                title = stringResource(MR.strings.migration_help_guide),
                icon = Icons.AutoMirrored.Outlined.HelpOutline,
                onClick = {
                    uriHandler.openUri("https://komikku-app.github.io/docs/guides/source-migration")
                },
            ),
        ),
        content = { contentPadding, _ ->
            // KMK -->
            if (state.showRustMove && state.rustMoves.isNotEmpty() && state.moveTitles.isNotEmpty()) {
                RustMoveDialog(
                    moves = state.rustMoves,
                    titles = state.moveTitles,
                    onDismiss = { screenModel.showRustMove(false) },
                    onMove = { chosen ->
                        screenModel.showRustMove(false)
                        screenModel.moveToRust(chosen) { context.toast(context.stringResource(KMR.strings.rust_move_done, it)) }
                    },
                )
            }
            // KMK <--
            MigrateSourceScreen(
                state = state,
                contentPadding = contentPadding,
                onClickItem = { source ->
                    navigator.push(MigrateMangaScreen(source.id))
                },
                onToggleSortingDirection = screenModel::toggleSortingDirection,
                onToggleSortingMode = screenModel::toggleSortingMode,
                // KMK -->
                onChangeSearchQuery = screenModel::search,
                // KMK <--
            )
        },
    )
}
