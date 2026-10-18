package eu.kanade.tachiyomi.ui.browse.migration.sources

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ExpandLess
import androidx.compose.material.icons.outlined.ExpandMore
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Checkbox
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TriStateCheckbox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.state.ToggleableState
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import tachiyomi.domain.manga.model.Manga
import tachiyomi.i18n.MR
import tachiyomi.i18n.kmk.KMR
import tachiyomi.presentation.core.i18n.stringResource

// KMK -->
/**
 * Which sources and which titles go to their Rust source. Every title starts ticked; a source row ticks or clears all of
 * its titles, and its arrow opens the list to pick one by one.
 */
@Composable
fun RustMoveDialog(
    moves: List<RustMove>,
    titles: Map<Long, List<Manga>>,
    onDismiss: () -> Unit,
    onMove: (Set<Long>) -> Unit,
) {
    var chosen by remember(titles) { mutableStateOf(titles.values.flatten().map { it.id }.toSet()) }
    var open by remember { mutableStateOf(emptySet<Long>()) }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(KMR.strings.rust_move_title)) },
        text = {
            LazyColumn(Modifier.heightIn(max = 460.dp)) {
                item { Text(stringResource(KMR.strings.rust_move_message), style = MaterialTheme.typography.bodyMedium) }
                moves.forEach { move ->
                    val own = titles[move.from.id].orEmpty()
                    val ids = own.map { it.id }.toSet()
                    val ticked = ids.count { it in chosen }
                    item(key = "source-${move.from.id}") {
                        Row(
                            Modifier.fillMaxWidth().padding(top = 12.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(4.dp),
                        ) {
                            TriStateCheckbox(
                                state = when (ticked) {
                                    0 -> ToggleableState.Off
                                    ids.size -> ToggleableState.On
                                    else -> ToggleableState.Indeterminate
                                },
                                onClick = { chosen = if (ticked == ids.size) chosen - ids else chosen + ids },
                            )
                            Text(
                                "${move.from.visualName}: $ticked/${ids.size} → ${move.to.name}",
                                Modifier.weight(1f),
                                style = MaterialTheme.typography.titleSmall,
                            )
                            IconButton(onClick = { open = if (move.from.id in open) open - move.from.id else open + move.from.id }) {
                                Icon(if (move.from.id in open) Icons.Outlined.ExpandLess else Icons.Outlined.ExpandMore, contentDescription = null)
                            }
                        }
                    }
                    if (move.from.id in open) {
                        items(own, key = { it.id }) { manga ->
                            Row(
                                Modifier.fillMaxWidth().clickable { chosen = if (manga.id in chosen) chosen - manga.id else chosen + manga.id }.padding(start = 24.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Checkbox(checked = manga.id in chosen, onCheckedChange = { chosen = if (it) chosen + manga.id else chosen - manga.id })
                                Text(manga.title, maxLines = 2, overflow = TextOverflow.Ellipsis, style = MaterialTheme.typography.bodyMedium)
                            }
                        }
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = { onMove(chosen) }, enabled = chosen.isNotEmpty()) {
                Text(stringResource(KMR.strings.rust_move_confirm_count, chosen.size))
            }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text(stringResource(MR.strings.action_cancel)) } },
    )
}
// KMK <--
