package eu.kanade.presentation.library.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import eu.kanade.presentation.manga.components.MangaCover
import tachiyomi.domain.library.model.LibraryManga
import tachiyomi.i18n.MR
import tachiyomi.i18n.kmk.KMR
import tachiyomi.presentation.core.components.CoverBadge
import tachiyomi.presentation.core.components.CoverProgress
import tachiyomi.presentation.core.components.SectionLabel
import tachiyomi.presentation.core.i18n.stringResource
import tachiyomi.presentation.core.theme.LocalCoverPalette
import tachiyomi.presentation.core.util.tilt3d
import tachiyomi.domain.manga.model.MangaCover as MangaCoverModel

/**
 * Redesign header of the library: eyebrow, large title with the number of titles and, when there is
 * something in progress, the "continue reading" card. Grids and the list render it as their first item.
 * Null means the classic layout.
 */
internal val LocalLibraryHeader = compositionLocalOf<(@Composable () -> Unit)?> { null }

@Composable
internal fun LibraryRedesignHeader(
    totalCount: Int,
    inProgress: LibraryManga?,
    onOpenManga: (Long) -> Unit,
    onContinueReading: ((LibraryManga) -> Unit)?,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 4.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            SectionLabel(text = stringResource(KMR.strings.redesign_library_eyebrow))
            Row(
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = stringResource(MR.strings.label_library),
                    style = MaterialTheme.typography.displaySmall,
                    letterSpacing = (-0.02).em,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f, fill = false),
                )
                if (totalCount > 0) CoverBadge(text = totalCount.toString())
            }
        }

        if (inProgress != null) {
            ContinueReadingCard(
                libraryManga = inProgress,
                onOpen = { onOpenManga(inProgress.manga.id) },
                onContinue = onContinueReading?.let { { it(inProgress) } },
            )
        }
    }
}

@Composable
private fun ContinueReadingCard(
    libraryManga: LibraryManga,
    onOpen: () -> Unit,
    onContinue: (() -> Unit)?,
) {
    val palette = LocalCoverPalette.current
    val manga = libraryManga.manga
    val shape = MaterialTheme.shapes.large
    val total = libraryManga.totalChapters.coerceAtLeast(1L)
    val progress = (libraryManga.readCount.toFloat() / total).coerceIn(0f, 1f)

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(MaterialTheme.colorScheme.surfaceContainerHigh)
            .border(1.dp, MaterialTheme.colorScheme.onSurface.copy(alpha = 0.12f), shape)
            .clickable(onClick = onOpen)
            .padding(12.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        MangaCover.Book(
            data = MangaCoverModel(
                mangaId = manga.id,
                sourceId = manga.source,
                isMangaFavorite = manga.favorite,
                ogUrl = manga.thumbnailUrl,
                lastModified = manga.coverLastModified,
            ),
            modifier = Modifier
                .width(88.dp)
                .tilt3d(maxDegrees = 15f, pressedScale = 1.04f),
            shape = MaterialTheme.shapes.small,
        )
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(
                text = stringResource(KMR.strings.redesign_continue_reading).uppercase(),
                color = palette.accent,
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold,
                letterSpacing = 0.14.em,
                maxLines = 1,
            )
            Text(
                text = manga.title,
                style = MaterialTheme.typography.titleLarge,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = stringResource(
                    KMR.strings.redesign_chapters_read_progress,
                    libraryManga.readCount.toInt(),
                    libraryManga.totalChapters.toInt(),
                ),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.72f),
                maxLines = 1,
            )
            CoverProgress(progress = progress)
            if (onContinue != null) {
                Button(
                    onClick = onContinue,
                    modifier = Modifier.heightIn(min = 44.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = palette.accent,
                        contentColor = palette.onAccent,
                    ),
                ) {
                    Text(
                        text = stringResource(KMR.strings.action_continue),
                        fontWeight = FontWeight.ExtraBold,
                    )
                }
            }
        }
    }
}
