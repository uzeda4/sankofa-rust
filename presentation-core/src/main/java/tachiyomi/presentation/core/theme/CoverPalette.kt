package tachiyomi.presentation.core.theme

import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

/**
 * Four tokens derived from a cover's seed color. Everything cover-tinted in the redesign reads from this
 * instead of picking colors on its own. See docs/redesign/HANDOFF.md, section 2.
 */
@Immutable
data class CoverPalette(
    /** Darkest tone, screen background. */
    val base: Color,
    /** Slightly lifted tone, side gradients and cards. */
    val surface: Color,
    /** Vibrant tone, primary action, active chips, progress. Always readable with [onAccent] on top. */
    val accent: Color,
    /** [accent] at low alpha, soft chip and badge background. */
    val veil: Color,
    /** Text/icon color to use on top of [accent]. */
    val onAccent: Color,
) {
    companion object {
        val Ink = Color(0xFF0F0E11)
        val DefaultAccent = Color(0xFFF2A65A)

        /** Minimum contrast between [onAccent] and [accent]. WCAG AA for normal text. */
        const val MIN_CONTRAST = 4.5f

        val Default: CoverPalette = fromSeed(DefaultAccent)

        fun fromSeed(seed: Color): CoverPalette {
            val (h, s, l) = seed.toHsl()
            // Neutral covers (gray, black, white) stay neutral instead of picking up a random hue.
            val neutral = s < NEUTRAL_SATURATION
            val accentSat = if (neutral) 0f else s.coerceIn(0.45f, 0.9f)

            var accentLight = l.coerceIn(0.55f, 0.75f)
            var accent = hsl(h, accentSat, accentLight)
            // onAccent is dark, so only a lighter accent can improve the contrast.
            while (contrastRatio(Ink, accent) < MIN_CONTRAST && accentLight < 0.95f) {
                accentLight += 0.02f
                accent = hsl(h, accentSat, accentLight)
            }

            val tintSat = if (neutral) 0f else s
            return CoverPalette(
                base = hsl(h, (tintSat * 0.6f).coerceIn(0f, 0.5f), 0.07f),
                surface = hsl(h, (tintSat * 0.5f).coerceIn(0f, 0.45f), 0.17f),
                accent = accent,
                veil = accent.copy(alpha = 0.18f),
                onAccent = Ink,
            )
        }

        private const val NEUTRAL_SATURATION = 0.08f
    }
}

val LocalCoverPalette = staticCompositionLocalOf { CoverPalette.Default }

/** WCAG contrast ratio between two opaque colors, 1f (same) to 21f (black on white). */
fun contrastRatio(a: Color, b: Color): Float {
    val la = a.luminance()
    val lb = b.luminance()
    return (max(la, lb) + 0.05f) / (min(la, lb) + 0.05f)
}

private data class Hsl(val h: Float, val s: Float, val l: Float)

private fun Color.toHsl(): Hsl {
    val r = red
    val g = green
    val b = blue
    val maxC = max(r, max(g, b))
    val minC = min(r, min(g, b))
    val delta = maxC - minC
    val l = (maxC + minC) / 2f
    if (delta == 0f) return Hsl(0f, 0f, l)
    val s = delta / (1f - abs(2f * l - 1f))
    val h = when (maxC) {
        r -> ((g - b) / delta).mod(6f)
        g -> (b - r) / delta + 2f
        else -> (r - g) / delta + 4f
    } * 60f
    return Hsl(h, s, l)
}

private fun hsl(h: Float, s: Float, l: Float): Color {
    val c = (1f - abs(2f * l - 1f)) * s
    val x = c * (1f - abs((h / 60f).mod(2f) - 1f))
    val m = l - c / 2f
    val (r, g, b) = when {
        h < 60f -> Triple(c, x, 0f)
        h < 120f -> Triple(x, c, 0f)
        h < 180f -> Triple(0f, c, x)
        h < 240f -> Triple(0f, x, c)
        h < 300f -> Triple(x, 0f, c)
        else -> Triple(c, 0f, x)
    }
    return Color(
        red = (r + m).coerceIn(0f, 1f),
        green = (g + m).coerceIn(0f, 1f),
        blue = (b + m).coerceIn(0f, 1f),
    )
}
