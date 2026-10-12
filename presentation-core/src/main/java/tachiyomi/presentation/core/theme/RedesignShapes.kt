package tachiyomi.presentation.core.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Shapes
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.unit.dp

/** Rounder corners than stock Material: covers 14 dp, cards 20 dp, sheets 32 dp. */
val RedesignShapes = Shapes(
    extraSmall = RoundedCornerShape(8.dp),
    small = RoundedCornerShape(14.dp),
    medium = RoundedCornerShape(20.dp),
    large = RoundedCornerShape(26.dp),
    extraLarge = RoundedCornerShape(32.dp),
)

/**
 * True when screens should draw the cover-tinted glow behind their content (the redesign look).
 * Off for the classic themes so they stay exactly as they were.
 */
val LocalCoverAtmosphere = staticCompositionLocalOf { false }
