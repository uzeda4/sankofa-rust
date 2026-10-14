package tachiyomi.presentation.core.theme

import androidx.compose.ui.graphics.Color
import io.kotest.matchers.floats.shouldBeGreaterThanOrEqual
import io.kotest.matchers.floats.shouldBeLessThan
import io.kotest.matchers.shouldBe
import org.junit.jupiter.api.Test
import kotlin.math.abs

class CoverPaletteTest {

    private val seeds = listOf(
        0xFF1B3A4B, // dark teal
        0xFF3B1F4A, // dark purple
        0xFF000000, // black
        0xFFFFFFFF, // white
        0xFFFF0000, // pure red
        0xFF0000FF, // pure blue
        0xFF808080, // gray
        0xFF7CF0C9, // mint
    ).map { Color(it) }

    @Test
    fun `accent is always readable with onAccent`() {
        seeds.forEach { seed ->
            val palette = CoverPalette.fromSeed(seed)
            contrastRatio(palette.onAccent, palette.accent) shouldBeGreaterThanOrEqual CoverPalette.MIN_CONTRAST
        }
    }

    @Test
    fun `neutral seed stays neutral`() {
        listOf(Color(0xFF000000), Color(0xFFFFFFFF), Color(0xFF808080)).forEach { seed ->
            val accent = CoverPalette.fromSeed(seed).accent
            abs(accent.red - accent.green) shouldBeLessThan 0.001f
            abs(accent.green - accent.blue) shouldBeLessThan 0.001f
        }
    }

    @Test
    fun `base is darker than surface and surface is darker than accent`() {
        seeds.forEach { seed ->
            val palette = CoverPalette.fromSeed(seed)
            val base = palette.base.luminanceOf()
            val surface = palette.surface.luminanceOf()
            val accent = palette.accent.luminanceOf()
            (surface >= base) shouldBe true
            (accent > surface) shouldBe true
        }
    }

    @Test
    fun `motion policy falls back when any limiter is on`() {
        policy(userReduce = true) shouldBe MotionPolicy.Reduced
        policy(animatorScale = 0f) shouldBe MotionPolicy.Reduced
        policy(powerSave = true) shouldBe MotionPolicy.Reduced
        policy(lowRam = true) shouldBe MotionPolicy.Reduced
    }

    @Test
    fun `motion policy keeps tilt but drops glass below API 31`() {
        val old = policy(sdkInt = 30)
        old.tilt shouldBe true
        old.parallax shouldBe true
        old.glass shouldBe false

        policy(sdkInt = 34) shouldBe MotionPolicy.Full
    }

    @Test
    fun `tilt intensity is clamped and zero turns tilt off`() {
        policy(tiltIntensity = 0.5f).tiltIntensity shouldBe 0.5f
        policy(tiltIntensity = 2f).tiltIntensity shouldBe 1f
        policy(tiltIntensity = 0f).tilt shouldBe false
        policy(tiltIntensity = 0f).parallax shouldBe true
    }

    private fun policy(
        userReduce: Boolean = false,
        animatorScale: Float = 1f,
        powerSave: Boolean = false,
        lowRam: Boolean = false,
        sdkInt: Int = 34,
        tiltIntensity: Float = 1f,
    ) = MotionPolicy.resolve(userReduce, animatorScale, powerSave, lowRam, sdkInt, tiltIntensity)

    private fun Color.luminanceOf(): Float = 0.2126f * red + 0.7152f * green + 0.0722f * blue
}
