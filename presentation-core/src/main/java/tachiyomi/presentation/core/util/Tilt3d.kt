package tachiyomi.presentation.core.util

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import kotlinx.coroutines.launch
import tachiyomi.presentation.core.theme.LocalMotionPolicy

private val TiltIn = spring<Float>(dampingRatio = Spring.DampingRatioNoBouncy, stiffness = Spring.StiffnessHigh)
private val TiltOut = spring<Float>(dampingRatio = 0.7f, stiffness = Spring.StiffnessLow)

/**
 * Tilts the element in 3D toward the finger while it is pressed, then springs back.
 * Observes pointers in the Final pass without consuming them, so clicks, long presses and scrolling
 * on the same element keep working. A pointer that was consumed by someone else (a scroll, a click) resets
 * the tilt, so a list item does not wobble while the list is scrolled under the finger.
 * Does nothing when [tilt][tachiyomi.presentation.core.theme.MotionPolicy.tilt] is off.
 *
 * @param maxDegrees rotation at the edge of the element. 11 for grids, 15 for the hero cover.
 * @param pressedScale scale while pressed.
 */
fun Modifier.tilt3d(maxDegrees: Float = 11f, pressedScale: Float = 1.05f): Modifier = composed {
    if (!LocalMotionPolicy.current.tilt) return@composed this

    val rotationX = remember { Animatable(0f) }
    val rotationY = remember { Animatable(0f) }
    val scale = remember { Animatable(1f) }
    val scope = rememberCoroutineScope()

    this
        .pointerInput(maxDegrees, pressedScale) {
            awaitPointerEventScope {
                while (true) {
                    val event = awaitPointerEvent(PointerEventPass.Final)
                    val change = event.changes.firstOrNull()
                    val w = size.width.toFloat()
                    val h = size.height.toFloat()
                    if (change != null && change.pressed && !change.isConsumed && w > 0f && h > 0f) {
                        val fx = (change.position.x / w).coerceIn(0f, 1f) - 0.5f
                        val fy = (change.position.y / h).coerceIn(0f, 1f) - 0.5f
                        scope.launch { rotationX.animateTo(-fy * 2f * maxDegrees, TiltIn) }
                        scope.launch { rotationY.animateTo(fx * 2f * maxDegrees, TiltIn) }
                        scope.launch { scale.animateTo(pressedScale, TiltIn) }
                    } else {
                        scope.launch { rotationX.animateTo(0f, TiltOut) }
                        scope.launch { rotationY.animateTo(0f, TiltOut) }
                        scope.launch { scale.animateTo(1f, TiltOut) }
                    }
                }
            }
        }
        .graphicsLayer {
            this.rotationX = rotationX.value
            this.rotationY = rotationY.value
            scaleX = scale.value
            scaleY = scale.value
            // Keeps the perspective subtle on high-density screens.
            cameraDistance = 12f * density
        }
}
