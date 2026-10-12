package tachiyomi.presentation.core.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import dev.chrisbanes.haze.HazeDefaults
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.HazeStyle
import dev.chrisbanes.haze.hazeEffect
import tachiyomi.presentation.core.theme.LocalCoverPalette
import tachiyomi.presentation.core.theme.LocalMotionPolicy

// Redesign building blocks. Colors come from LocalCoverPalette (see docs/redesign/HANDOFF.md, section 3), so
// everything follows the cover that is in focus. Fonts stay on the Material typography until the redesign
// typefaces are added.

/** Filter/category chip. 44 dp tall, filled with the cover accent when [selected]. */
@Composable
fun CoverChip(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val palette = LocalCoverPalette.current
    val onSurface = MaterialTheme.colorScheme.onSurface
    val container by animateColorAsState(
        targetValue = if (selected) palette.accent else onSurface.copy(alpha = 0.08f),
        label = "CoverChipContainer",
    )
    val border = if (selected) palette.accent else onSurface.copy(alpha = 0.16f)
    Box(
        modifier = modifier
            .heightIn(min = 44.dp)
            .clip(CircleShape)
            .background(container)
            .border(1.dp, border, CircleShape)
            .selectable(selected = selected, role = Role.Tab, onClick = onClick)
            .padding(horizontal = 18.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            color = if (selected) palette.onAccent else onSurface,
            style = MaterialTheme.typography.labelLarge,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

/** Small counter on top of a cover, for example the number of unread chapters. */
@Composable
fun CoverBadge(
    text: String,
    modifier: Modifier = Modifier,
) {
    val palette = LocalCoverPalette.current
    Box(
        modifier = modifier
            .heightIn(min = 24.dp)
            .widthIn(min = 24.dp)
            .clip(CircleShape)
            .background(palette.accent)
            .padding(horizontal = 8.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = text,
            color = palette.onAccent,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
        )
    }
}

/** Section header: small, upper case, wide tracking. */
@Composable
fun SectionLabel(
    text: String,
    modifier: Modifier = Modifier,
) {
    Text(
        text = text.uppercase(),
        modifier = modifier,
        color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.72f),
        style = MaterialTheme.typography.labelSmall,
        fontWeight = FontWeight.Bold,
        letterSpacing = 0.14.em,
        maxLines = 1,
    )
}

/** Thin progress bar in the cover accent. [progress] is clamped to 0..1. */
@Composable
fun CoverProgress(
    progress: Float,
    modifier: Modifier = Modifier,
) {
    val palette = LocalCoverPalette.current
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(6.dp)
            .clip(CircleShape)
            .background(MaterialTheme.colorScheme.onSurface.copy(alpha = 0.14f)),
    ) {
        Box(
            modifier = Modifier
                .fillMaxHeight()
                .fillMaxWidth(progress.coerceIn(0f, 1f))
                .clip(CircleShape)
                .background(palette.accent),
        )
    }
}

/**
 * Haze state of the content that sits behind glass surfaces. A screen creates one with `HazeState()`, marks
 * its scrolling content with `Modifier.hazeSource(state)` and provides it here.
 */
val LocalGlassBackdrop = staticCompositionLocalOf<HazeState?> { null }

/**
 * Surface for dock, reader bars and sheets (depth level Z2), with a hairline border.
 * With a [hazeState] and `MotionPolicy.glass` it blurs what is behind it. Otherwise (no backdrop provided,
 * API below 31, power save, reduced motion) it is a nearly opaque solid, which keeps text readable.
 */
@Composable
fun GlassSurface(
    modifier: Modifier = Modifier,
    shape: Shape = RoundedCornerShape(26.dp),
    hazeState: HazeState? = LocalGlassBackdrop.current,
    content: @Composable BoxScope.() -> Unit,
) {
    val container = MaterialTheme.colorScheme.surfaceContainerHigh
    val surface = if (hazeState != null && LocalMotionPolicy.current.glass) {
        Modifier
            .clip(shape)
            .hazeEffect(
                state = hazeState,
                style = HazeStyle(
                    backgroundColor = Color.Transparent,
                    tint = HazeDefaults.tint(container.copy(alpha = 0.6f)),
                    blurRadius = 24.dp,
                ),
            )
    } else {
        Modifier
            .clip(shape)
            .background(container.copy(alpha = 0.92f))
    }
    Box(
        modifier = modifier
            .then(surface)
            .border(1.dp, MaterialTheme.colorScheme.onSurface.copy(alpha = 0.14f), shape),
        content = content,
    )
}

/**
 * Placeholder fill that sweeps a soft highlight, for content that is still loading.
 * Static when the motion policy turns decorative animation off.
 */
fun Modifier.shimmer(): Modifier = composed {
    val onSurface = MaterialTheme.colorScheme.onSurface
    val base = onSurface.copy(alpha = 0.06f)
    val peak = onSurface.copy(alpha = 0.18f)
    if (!LocalMotionPolicy.current.parallax) return@composed background(base)

    val sweep by rememberInfiniteTransition(label = "Shimmer").animateFloat(
        initialValue = -1f,
        targetValue = 2f,
        animationSpec = infiniteRepeatable(tween(durationMillis = 1500, easing = LinearEasing)),
        label = "ShimmerSweep",
    )
    drawBehind {
        drawRect(
            Brush.horizontalGradient(
                colors = listOf(base, peak, base),
                startX = (sweep - 0.5f) * size.width,
                endX = (sweep + 0.5f) * size.width,
            ),
        )
    }
}
