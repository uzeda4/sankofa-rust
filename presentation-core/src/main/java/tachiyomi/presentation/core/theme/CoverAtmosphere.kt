package tachiyomi.presentation.core.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance

/**
 * Draws the redesign "atmosphere": a soft glow of the cover accent from the top left and a lifted
 * surface tone on the right. Drawn over the content with low alpha, so it works on top of screens that
 * paint their own opaque background. No-op unless [LocalCoverAtmosphere] is on.
 */
fun Modifier.coverAtmosphere(): Modifier = composed {
    if (!LocalCoverAtmosphere.current) return@composed this
    val palette = LocalCoverPalette.current
    val isDark = MaterialTheme.colorScheme.background.luminance() < 0.5f
    drawWithContent {
        drawContent()
        drawRect(
            brush = Brush.radialGradient(
                colors = listOf(
                    palette.accent.copy(alpha = if (isDark) 0.20f else 0.14f),
                    Color.Transparent,
                ),
                center = Offset(0f, 0f),
                radius = size.maxDimension * 0.75f,
            ),
        )
        // The lifted surface tone only makes sense on a dark ground; on paper it would read as a smudge.
        if (isDark) {
            drawRect(
                brush = Brush.radialGradient(
                    colors = listOf(palette.surface.copy(alpha = 0.45f), Color.Transparent),
                    center = Offset(size.width, size.height * 0.5f),
                    radius = size.maxDimension * 0.6f,
                ),
            )
        }
    }
}
