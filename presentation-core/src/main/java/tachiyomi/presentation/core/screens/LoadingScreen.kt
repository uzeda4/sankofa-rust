package tachiyomi.presentation.core.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import tachiyomi.i18n.MR
import tachiyomi.presentation.core.components.shimmer
import tachiyomi.presentation.core.i18n.stringResource
import tachiyomi.presentation.core.theme.LocalCoverAtmosphere

@Composable
fun LoadingScreen(modifier: Modifier = Modifier) {
    // KMK -->
    if (LocalCoverAtmosphere.current) {
        // Redesign: placeholder cards that shimmer instead of a lone spinner.
        val description = stringResource(MR.strings.loading)
        Column(
            modifier = modifier
                .fillMaxSize()
                .padding(horizontal = 16.dp, vertical = 12.dp)
                .semantics { contentDescription = description },
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            repeat(SKELETON_ROWS) { index ->
                Box(
                    modifier = Modifier
                        .fillMaxWidth(if (index % 3 == 2) 0.8f else 1f)
                        .height(72.dp)
                        .clip(SkeletonShape)
                        .shimmer(),
                )
            }
        }
        return
    }
    // KMK <--
    Box(
        modifier = modifier.fillMaxSize(),
        contentAlignment = Alignment.Center,
    ) {
        CircularProgressIndicator()
    }
}

// KMK -->
private const val SKELETON_ROWS = 6
private val SkeletonShape = RoundedCornerShape(20.dp)
// KMK <--
