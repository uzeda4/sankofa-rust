package tachiyomi.presentation.core.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance

/**
 * Draws the redesign "atmosphere": a soft glow of the cover accent from the top left and a lifted
 * surface tone on the right. It is drawn behind the content, so covers and text are never tinted.
 * Applied by the shared Scaffold, on top of the screen background color.
 * No-op unless [LocalCoverAtmosphere] is on.
 */
fun Modifier.coverAtmosphere(): Modifier = composed {
    if (!LocalCoverAtmosphere.current) return@composed this
    val palette = LocalCoverPalette.current
    val isDark = MaterialTheme.colorScheme.background.luminance() < 0.5f
    drawBehind {
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
