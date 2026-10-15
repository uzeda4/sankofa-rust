package tachiyomi.presentation.core.util

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import tachiyomi.presentation.core.theme.LocalMotionPolicy

/** Set by the root navigator host; null where there is no screen transition to share elements across. */
val LocalSharedTransitionScope = staticCompositionLocalOf<SharedTransitionScope?> { null }

/** Visibility scope of the screen currently composed by the navigator. */
val LocalScreenVisibilityScope = staticCompositionLocalOf<AnimatedVisibilityScope?> { null }

/**
 * Id of the title whose cover was tapped last. Only that cover takes part in the shared transition, so a title
 * that appears twice on screen (two categories, a list and a grid) never registers the same key twice.
 */
object SharedCoverTarget {
    var mangaId: Long? by mutableStateOf(null)
}

/**
 * Makes this cover fly between screens: the library tile it was tapped from and the cover of the detail screen.
 * [isOrigin] is true for tiles, which only take part while they are the tapped one; the detail cover passes false.
 */
@Composable
fun Modifier.sharedCover(mangaId: Long, isOrigin: Boolean = true): Modifier {
    val scope = LocalSharedTransitionScope.current
    val visibility = LocalScreenVisibilityScope.current
    if (scope == null || visibility == null || !LocalMotionPolicy.current.parallax) return this
    if (isOrigin && SharedCoverTarget.mangaId != mangaId) return this
    return with(scope) {
        sharedElement(rememberSharedContentState(key = "cover-$mangaId"), visibility)
    }
}
