package tachiyomi.presentation.core.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.unit.Dp
import tachiyomi.presentation.core.theme.LocalCoverPalette

// Same drawing as the launcher icon (app/src/main/res/drawable/ic_launcher_foreground.xml), in a 100 unit box.
private const val TAIL_BACK = "M46,63C37,60 31,52 31,42C40,45 46,52 49,59Z"
private const val TAIL_FRONT = "M45,66C34,67 23,62 18,52C30,51 40,56 48,62Z"
private const val BODY = "M79,66a20,14.5 -10 1 1 -40,0a20,14.5 -10 1 1 40,0z"
private const val NECK = "M68,58C81,48 79,30 64,28C58,27 53,29 50,33"
private const val HEAD = "M42,35a8,8 0 1 0 16,0a8,8 0 1 0 -16,0z"
private const val BEAK = "M44,30C38,31 32,34 29,39C35,40 40,42 44,43Z"
private const val WING = "M50,66C56,74 68,73 75,64"
private const val LEGS = "M54,80L52,92M64,79L66,91"

/**
 * The Sankofa bird: it looks back over its body and carries an egg. The body takes [bodyColor], the egg
 * [eggColor] (the cover accent by default) and the cut-outs (wing line, egg outline, eye) [cutoutColor], which
 * should be the color behind the mark.
 */
@Composable
fun BrandMark(
    size: Dp,
    modifier: Modifier = Modifier,
    bodyColor: Color = MaterialTheme.colorScheme.onSurface,
    eggColor: Color = LocalCoverPalette.current.accent,
    cutoutColor: Color = MaterialTheme.colorScheme.surface,
) {
    val paths = remember {
        listOf(TAIL_BACK, TAIL_FRONT, BODY, HEAD, BEAK).map { PathParser().parsePathString(it).toPath() }
    }
    val neck = remember { PathParser().parsePathString(NECK).toPath() }
    val wing = remember { PathParser().parsePathString(WING).toPath() }
    val legs = remember { PathParser().parsePathString(LEGS).toPath() }

    Canvas(modifier = modifier.size(size)) {
        withTransform({
            val factor = this@Canvas.size.width / 100f
            scale(factor, factor, pivot = Offset.Zero)
            // The drawing sits low in its box; lift it so it is centered.
            translate(top = -7f)
        }) {
            paths.forEach { drawPath(it, bodyColor) }
            drawPath(neck, bodyColor, style = Stroke(width = 9f, cap = StrokeCap.Round))
            drawPath(wing, cutoutColor, style = Stroke(width = 2.2f, cap = StrokeCap.Round))
            drawPath(legs, bodyColor, style = Stroke(width = 3.6f, cap = StrokeCap.Round))
            egg(eggColor, cutoutColor)
            drawCircle(cutoutColor, radius = 2.1f, center = Offset(51f, 33.5f))
        }
    }
}

private fun DrawScope.egg(fill: Color, outline: Color) {
    val center = Offset(29f, 45f)
    drawCircle(fill, radius = 7.5f, center = center)
    drawCircle(outline, radius = 7.5f, center = center, style = Stroke(width = 2.2f))
}
