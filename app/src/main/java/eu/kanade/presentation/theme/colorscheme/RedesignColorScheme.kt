package eu.kanade.presentation.theme.colorscheme

import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color

/**
 * Redesign theme. Dark-first: warm ink ground, paper text, amber accent.
 * Tokens come from docs/redesign/HANDOFF.md, section 2. When a cover is in focus the cover-based
 * theme takes over, so this is the "neutral by default" state.
 */
internal object RedesignColorScheme : BaseColorScheme() {

    override val darkScheme = darkColorScheme(
        primary = Color(0xFFF2A65A),
        onPrimary = Color(0xFF0F0E11),
        primaryContainer = Color(0xFF4A3015),
        onPrimaryContainer = Color(0xFFFFDDB8),
        inversePrimary = Color(0xFF8A5A1E),
        secondary = Color(0xFFF2A65A), // Unread badge
        onSecondary = Color(0xFF0F0E11), // Unread badge text
        secondaryContainer = Color(0xFF3A2B1B), // Navigation bar selector pill & progress indicator (remaining)
        onSecondaryContainer = Color(0xFFF2A65A), // Navigation bar selector icon
        tertiary = Color(0xFF7CF0C9), // Downloaded badge
        onTertiary = Color(0xFF0F0E11), // Downloaded badge text
        tertiaryContainer = Color(0xFF123D31),
        onTertiaryContainer = Color(0xFFB8F7E3),
        background = Color(0xFF0F0E11),
        onBackground = Color(0xFFF4F0EA),
        surface = Color(0xFF0F0E11),
        onSurface = Color(0xFFF4F0EA),
        surfaceVariant = Color(0xFF1E1B22), // Navigation bar background (ThemePrefWidget)
        onSurfaceVariant = Color(0xFFBDB8B1),
        surfaceTint = Color(0xFFF2A65A),
        inverseSurface = Color(0xFFF4F0EA),
        inverseOnSurface = Color(0xFF0F0E11),
        outline = Color(0xFF6B666F),
        outlineVariant = Color(0xFF3A363F),
        error = Color(0xFFFF8A80),
        onError = Color(0xFF0F0E11),
        errorContainer = Color(0xFF5A1E1A),
        onErrorContainer = Color(0xFFFFDAD6),
        surfaceContainerLowest = Color(0xFF0A090C),
        surfaceContainerLow = Color(0xFF151318),
        surfaceContainer = Color(0xFF1A171E), // Navigation bar background
        surfaceContainerHigh = Color(0xFF211E26),
        surfaceContainerHighest = Color(0xFF2A262F),
    )

    override val lightScheme = lightColorScheme(
        primary = Color(0xFF9A5B14),
        onPrimary = Color(0xFFFFFFFF),
        primaryContainer = Color(0xFFF7D9B0),
        onPrimaryContainer = Color(0xFF3A2000),
        inversePrimary = Color(0xFFF2A65A),
        secondary = Color(0xFF9A5B14), // Unread badge
        onSecondary = Color(0xFFFFFFFF), // Unread badge text
        secondaryContainer = Color(0xFFF0D9B8), // Navigation bar selector pill & progress indicator (remaining)
        onSecondaryContainer = Color(0xFF3A2000), // Navigation bar selector icon
        tertiary = Color(0xFF0B6B52), // Downloaded badge
        onTertiary = Color(0xFFFFFFFF), // Downloaded badge text
        tertiaryContainer = Color(0xFFB8F7E3),
        onTertiaryContainer = Color(0xFF00382A),
        background = Color(0xFFF4F0EA),
        onBackground = Color(0xFF0F0E11),
        surface = Color(0xFFF4F0EA),
        onSurface = Color(0xFF0F0E11),
        surfaceVariant = Color(0xFFE9E3D9), // Navigation bar background (ThemePrefWidget)
        onSurfaceVariant = Color(0xFF4A4640),
        surfaceTint = Color(0xFF9A5B14),
        inverseSurface = Color(0xFF1A171E),
        inverseOnSurface = Color(0xFFF4F0EA),
        outline = Color(0xFF7A736B),
        outlineVariant = Color(0xFFCFC7BB),
        error = Color(0xFFB3261E),
        onError = Color(0xFFFFFFFF),
        errorContainer = Color(0xFFF9DEDC),
        onErrorContainer = Color(0xFF410E0B),
        surfaceContainerLowest = Color(0xFFFFFFFF),
        surfaceContainerLow = Color(0xFFF9F6F1),
        surfaceContainer = Color(0xFFEFEAE1), // Navigation bar background
        surfaceContainerHigh = Color(0xFFE7E1D6),
        surfaceContainerHighest = Color(0xFFDFD8CC),
    )
}
