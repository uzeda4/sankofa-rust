package tachiyomi.presentation.core.theme

import android.app.ActivityManager
import android.content.Context
import android.os.Build
import android.os.PowerManager
import android.provider.Settings
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.platform.LocalContext

/**
 * Which motion effects the redesign may use right now. Every 3D, parallax or blur effect must check this
 * before running, so the fallback described in docs/redesign/HANDOFF.md (section 4.2) lives in one place.
 */
@Immutable
data class MotionPolicy(
    /** Pointer-driven 3D tilt of covers. */
    val tilt: Boolean,
    /** Layer parallax inside covers and hero images. */
    val parallax: Boolean,
    /** Blurred glass surfaces. Needs RenderEffect (API 31+). */
    val glass: Boolean,
    /** Multiplier for the tilt angle, 0f to 1f. The user can soften it without turning motion off. */
    val tiltIntensity: Float = 1f,
) {
    companion object {
        val Full = MotionPolicy(tilt = true, parallax = true, glass = true)
        val Reduced = MotionPolicy(tilt = false, parallax = false, glass = false, tiltIntensity = 0f)

        fun resolve(
            userReduce: Boolean,
            animatorScale: Float,
            powerSave: Boolean,
            lowRam: Boolean,
            sdkInt: Int,
            tiltIntensity: Float = 1f,
        ): MotionPolicy = when {
            userReduce || animatorScale == 0f || powerSave || lowRam -> Reduced
            else -> {
                val intensity = tiltIntensity.coerceIn(0f, 1f)
                Full.copy(
                    tilt = intensity > 0f,
                    glass = sdkInt >= Build.VERSION_CODES.S,
                    tiltIntensity = intensity,
                )
            }
        }
    }
}

val LocalMotionPolicy = staticCompositionLocalOf { MotionPolicy.Full }

/**
 * Reads the system state once per composition scope.
 * Power-save and animator-scale changes are picked up the next time the theme is recomposed, which is fine
 * for a policy that only gates decoration.
 */
@Composable
fun rememberMotionPolicy(userReduce: Boolean = false, tiltIntensity: Float = 1f): MotionPolicy {
    val context = LocalContext.current
    return remember(userReduce, tiltIntensity) { context.currentMotionPolicy(userReduce, tiltIntensity) }
}

private fun Context.currentMotionPolicy(userReduce: Boolean, tiltIntensity: Float): MotionPolicy {
    val scale = Settings.Global.getFloat(contentResolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f)
    val powerSave = getSystemService(PowerManager::class.java)?.isPowerSaveMode == true
    val lowRam = getSystemService(ActivityManager::class.java)?.isLowRamDevice == true
    return MotionPolicy.resolve(userReduce, scale, powerSave, lowRam, Build.VERSION.SDK_INT, tiltIntensity)
}
