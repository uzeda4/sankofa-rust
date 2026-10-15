package tachiyomi.presentation.core.util

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.addOutline
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.Dp
import kotlinx.coroutines.launch
import tachiyomi.presentation.core.theme.LocalMotionPolicy
import kotlin.math.max

private val TiltIn = spring<Float>(dampingRatio = Spring.DampingRatioNoBouncy, stiffness = Spring.StiffnessHigh)
private val TiltOut = spring<Float>(dampingRatio = 0.7f, stiffness = Spring.StiffnessLow)

/** Peak opacity of the glare highlight that follows the finger. */
private const val GLARE_ALPHA = 0.26f

/**
 * Pointer state of a tilting element, normalized to -1..1 on each axis (0 = center / at rest).
 * Share one between a cover and the layers around it ([tiltParallax]) so they move together.
 */
class TiltState {
    internal val nx = Animatable(0f)
    internal val ny = Animatable(0f)

    /** 0 at rest, 1 while pressed. */
    internal val press = Animatable(0f)

    val x: Float get() = nx.value
    val y: Float get() = ny.value
}

/** Tilt state of the cover on the current screen, for background layers that should drift with it. */
val LocalCoverTilt = staticCompositionLocalOf<TiltState?> { null }

/**
 * Tilts the element in 3D toward the finger while it is pressed, then springs back, and sweeps a soft glare
 * highlight across it. Observes pointers in the Final pass without consuming them, so clicks, long presses and
 * scrolling on the same element keep working. A pointer that was consumed by someone else (a scroll, a click)
 * resets the tilt, so a list item does not wobble while the list is scrolled under the finger.
 * Does nothing when [tilt][tachiyomi.presentation.core.theme.MotionPolicy.tilt] is off.
 *
 * @param maxDegrees rotation at the edge of the element. 11 for grids, 15 for the hero cover.
 * @param pressedScale scale while pressed.
 * @param state pass a shared [TiltState] to drive other layers ([tiltParallax]) from the same gesture.
 * @param glareShape clip of the glare, the corner shape of the element. Null for the theme's small shape.
 */
fun Modifier.tilt3d(
    maxDegrees: Float = 11f,
    pressedScale: Float = 1.05f,
    state: TiltState? = null,
    glareShape: Shape? = null,
): Modifier = composed {
    val policy = LocalMotionPolicy.current
    if (!policy.tilt) return@composed this
    // The user's intensity setting scales both the angle and the press zoom.
    val degrees = maxDegrees * policy.tiltIntensity
    val zoom = (pressedScale - 1f) * policy.tiltIntensity

    val tilt = state ?: remember { TiltState() }
    val scope = rememberCoroutineScope()
    val shape = glareShape ?: MaterialTheme.shapes.small

    this
        .pointerInput(tilt) {
            awaitPointerEventScope {
                while (true) {
                    val event = awaitPointerEvent(PointerEventPass.Final)
                    val change = event.changes.firstOrNull()
                    val w = size.width.toFloat()
                    val h = size.height.toFloat()
                    if (change != null && change.pressed && !change.isConsumed && w > 0f && h > 0f) {
                        val fx = (change.position.x / w).coerceIn(0f, 1f) * 2f - 1f
                        val fy = (change.position.y / h).coerceIn(0f, 1f) * 2f - 1f
                        scope.launch { tilt.nx.animateTo(fx, TiltIn) }
                        scope.launch { tilt.ny.animateTo(fy, TiltIn) }
                        scope.launch { tilt.press.animateTo(1f, TiltIn) }
                    } else {
                        scope.launch { tilt.nx.animateTo(0f, TiltOut) }
                        scope.launch { tilt.ny.animateTo(0f, TiltOut) }
                        scope.launch { tilt.press.animateTo(0f, TiltOut) }
                    }
                }
            }
        }
        .graphicsLayer {
            rotationX = -tilt.ny.value * degrees
            rotationY = tilt.nx.value * degrees
            val s = 1f + zoom * tilt.press.value
            scaleX = s
            scaleY = s
            // Keeps the perspective subtle on high-density screens.
            cameraDistance = 12f * density
        }
        .drawWithContent {
            drawContent()
            val p = tilt.press.value
            if (p > 0.01f) {
                val outline = shape.createOutline(size, layoutDirection, this)
                val brush = Brush.radialGradient(
                    colors = listOf(Color.White.copy(alpha = GLARE_ALPHA * p), Color.Transparent),
                    center = Offset((tilt.nx.value + 1f) / 2f * size.width, (tilt.ny.value + 1f) / 2f * size.height),
                    radius = max(size.width, size.height) * 0.8f,
                )
                clipPath(Path().apply { addOutline(outline) }) { drawRect(brush) }
            }
        }
}

/**
 * Moves a background layer opposite to the cover's tilt, up to [maxShift] on each axis, so the cover reads as
 * floating in front of it. Needs the [TiltState] the cover uses (usually [LocalCoverTilt]); no-op without one.
 */
@Composable
fun Modifier.tiltParallax(maxShift: Dp, state: TiltState? = LocalCoverTilt.current): Modifier {
    if (state == null || !LocalMotionPolicy.current.parallax) return this
    return graphicsLayer {
        val shift = maxShift.toPx()
        translationX = -state.nx.value * shift
        translationY = -state.ny.value * shift
    }
}
