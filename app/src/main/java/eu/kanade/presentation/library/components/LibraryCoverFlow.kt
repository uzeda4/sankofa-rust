package eu.kanade.presentation.library.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PageSize
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.zIndex
import eu.kanade.presentation.manga.components.MangaCover
import tachiyomi.domain.library.model.LibraryManga
import tachiyomi.i18n.kmk.KMR
import tachiyomi.presentation.core.components.SectionLabel
import tachiyomi.presentation.core.i18n.stringResource
import tachiyomi.presentation.core.theme.LocalMotionPolicy
import tachiyomi.presentation.core.util.SharedCoverTarget
import tachiyomi.presentation.core.util.sharedCover
import kotlin.math.abs
import tachiyomi.domain.manga.model.MangaCover as MangaCoverModel

private val CoverWidth = 112.dp
private const val FLOW_DEGREES = 42f

/**
 * "Rediscover": unread titles from the library laid out as a cover flow. The centered cover faces front, the
 * ones on its sides turn away in 3D and shrink; tapping the centered one opens it. The Sankofa idea of going
 * back to fetch what was left behind. Falls back to a flat carousel when 3D motion is off.
 */
@Composable
internal fun RediscoverCoverFlow(
    items: List<LibraryManga>,
    onOpenManga: (Long) -> Unit,
    modifier: Modifier = Modifier,
) {
    if (items.isEmpty()) return
    val policy = LocalMotionPolicy.current
    val pagerState = rememberPagerState { items.size }

    Column(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            SectionLabel(text = stringResource(KMR.strings.redesign_eyebrow_rediscover))
            Text(
                text = stringResource(KMR.strings.redesign_rediscover),
                style = MaterialTheme.typography.titleLarge,
                letterSpacing = (-0.01).em,
            )
        }

        BoxWithConstraints(modifier = Modifier.fillMaxWidth()) {
            val sidePadding = ((maxWidth - CoverWidth) / 2).coerceAtLeast(0.dp)
            HorizontalPager(
                state = pagerState,
                pageSize = PageSize.Fixed(CoverWidth),
                // Neighbours tuck behind the centered cover.
                pageSpacing = (-24).dp,
                contentPadding = PaddingValues(horizontal = sidePadding),
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth(),
            ) { page ->
                val manga = items[page].manga
                CoverFlowPage(
                    manga = manga,
                    offset = { (pagerState.currentPage - page) + pagerState.currentPageOffsetFraction },
                    flat = !policy.tilt,
                    onClick = {
                        if (pagerState.currentPage == page) {
                            SharedCoverTarget.mangaId = manga.id
                            onOpenManga(manga.id)
                        }
                    },
                )
            }
        }

        val current = items[pagerState.currentPage.coerceIn(0, items.lastIndex)].manga
        Text(
            text = current.title,
            style = MaterialTheme.typography.titleSmall,
            textAlign = TextAlign.Center,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier
                .fillMaxWidth()
                .widthIn(max = 320.dp)
                .padding(horizontal = 16.dp),
        )
    }
}

@Composable
private fun CoverFlowPage(
    manga: tachiyomi.domain.manga.model.Manga,
    offset: () -> Float,
    flat: Boolean,
    onClick: () -> Unit,
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
            .width(CoverWidth)
            .zIndex(-abs(offset()))
            .graphicsLayer {
                val o = offset().coerceIn(-2f, 2f)
                val a = abs(o).coerceAtMost(1f)
                // Covers on the left face right and the ones on the right face left, like a shelf.
                rotationY = if (flat) 0f else o.coerceIn(-1f, 1f) * FLOW_DEGREES
                val s = 1f - 0.18f * a
                scaleX = s
                scaleY = s
                alpha = 1f - 0.4f * (abs(o) / 2f)
                cameraDistance = 14f * density
            }
            .sharedCover(manga.id)
            .clip(MaterialTheme.shapes.small)
            .clickable(onClick = onClick),
        shape = MaterialTheme.shapes.small,
    )
}
