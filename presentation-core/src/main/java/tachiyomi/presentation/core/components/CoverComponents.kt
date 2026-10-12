package tachiyomi.presentation.core.components

import androidx.compose.animation.animateColorAsState
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import tachiyomi.presentation.core.theme.LocalCoverPalette

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
 * Surface for dock, reader bars and sheets (depth level Z2).
 * Translucent with a hairline border. It stays at 92% opacity because there is no backdrop blur yet;
 * drop it to ~62% only once a blur (for example Haze) sits behind it and `MotionPolicy.glass` is on.
 */
@Composable
fun GlassSurface(
    modifier: Modifier = Modifier,
    shape: Shape = RoundedCornerShape(26.dp),
    content: @Composable BoxScope.() -> Unit,
) {
    Box(
        modifier = modifier
            .clip(shape)
            .background(MaterialTheme.colorScheme.surfaceContainerHigh.copy(alpha = 0.92f))
            .border(1.dp, MaterialTheme.colorScheme.onSurface.copy(alpha = 0.14f), shape),
        content = content,
    )
}
