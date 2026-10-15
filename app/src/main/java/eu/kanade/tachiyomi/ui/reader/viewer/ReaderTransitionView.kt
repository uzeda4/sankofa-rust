package eu.kanade.tachiyomi.ui.reader.viewer

import android.content.Context
import android.util.AttributeSet
import android.view.ViewTreeObserver
import androidx.annotation.ColorInt
import androidx.compose.foundation.layout.Box
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.AbstractComposeView
import eu.kanade.domain.ui.UiPreferences
import eu.kanade.presentation.reader.ChapterTransition
import eu.kanade.presentation.theme.TachiyomiTheme
import eu.kanade.tachiyomi.data.download.DownloadManager
import eu.kanade.tachiyomi.ui.reader.model.ChapterTransition
import tachiyomi.domain.manga.model.Manga
import tachiyomi.presentation.core.theme.LocalMotionPolicy
import tachiyomi.source.local.isLocal
import uy.kohesive.injekt.Injekt
import uy.kohesive.injekt.api.get
import kotlin.math.abs

class ReaderTransitionView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    // KMK -->
    @ColorInt private val seedColor: Int? = null,
    // KMK <--
) :
    AbstractComposeView(context, attrs) {

    private var data: Data? by mutableStateOf(null)

    // KMK -->
    // Where the card sits relative to the screen center, -1..1 on each axis. While the user swipes to the next
    // chapter the card turns like a page: it rotates in 3D and fades as it moves away from the center.
    private var turnX by mutableFloatStateOf(0f)
    private var turnY by mutableFloatStateOf(0f)
    private val location = IntArray(2)
    private val turnListener = ViewTreeObserver.OnPreDrawListener {
        updateTurn()
        true
    }
    // KMK <--

    init {
        layoutParams = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT)
    }

    // KMK -->
    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        viewTreeObserver.addOnPreDrawListener(turnListener)
    }

    override fun onDetachedFromWindow() {
        if (viewTreeObserver.isAlive) viewTreeObserver.removeOnPreDrawListener(turnListener)
        super.onDetachedFromWindow()
    }

    private fun updateTurn() {
        val root = rootView
        if (root.width == 0 || root.height == 0 || width == 0) return
        getLocationInWindow(location)
        val x = ((location[0] + width / 2f) / root.width * 2f - 1f).coerceIn(-1f, 1f)
        val y = ((location[1] + height / 2f) / root.height * 2f - 1f).coerceIn(-1f, 1f)
        // Only write on a visible change so the draw phase is not invalidated every frame.
        if (abs(x - turnX) > TURN_EPSILON) turnX = x
        if (abs(y - turnY) > TURN_EPSILON) turnY = y
    }
    // KMK <--

    fun bind(transition: ChapterTransition, downloadManager: DownloadManager, manga: Manga?) {
        data = if (manga != null) {
            Data(
                transition = transition,
                currChapterDownloaded = transition.from.pageLoader?.isLocal == true,
                goingToChapterDownloaded = manga.isLocal() ||
                    transition.to?.chapter?.let { goingToChapter ->
                        downloadManager.isChapterDownloaded(
                            chapterName = goingToChapter.name,
                            chapterScanlator = goingToChapter.scanlator,
                            chapterUrl = goingToChapter.url,
                            mangaTitle = /* SY --> */ manga.ogTitle, /* SY <-- */
                            sourceId = manga.source,
                            skipCache = true,
                        )
                    } ?: false,
            )
        } else {
            null
        }
    }

    @Composable
    override fun Content() {
        data?.let {
            // KMK -->
            val uiPreferences = Injekt.get<UiPreferences>()
            val themeCoverBased = uiPreferences.themeCoverBased().get()
            // KMK <--
            TachiyomiTheme(
                // KMK -->
                seedColor = seedColor?.let { Color(seedColor) }.takeIf { themeCoverBased },
                // KMK <--
            ) {
                CompositionLocalProvider(
                    LocalTextStyle provides MaterialTheme.typography.bodySmall,
                    LocalContentColor provides MaterialTheme.colorScheme.onBackground,
                ) {
                    // KMK -->
                    val policy = LocalMotionPolicy.current
                    Box(
                        modifier = Modifier.graphicsLayer {
                            if (policy.tilt) {
                                rotationY = -turnX * TURN_DEGREES_X * policy.tiltIntensity
                                rotationX = turnY * TURN_DEGREES_Y * policy.tiltIntensity
                                alpha = 1f - TURN_FADE * maxOf(abs(turnX), abs(turnY))
                                cameraDistance = 14f * density
                            }
                        },
                    ) {
                        // KMK <--
                        ChapterTransition(
                            transition = it.transition,
                            currChapterDownloaded = it.currChapterDownloaded,
                            goingToChapterDownloaded = it.goingToChapterDownloaded,
                        )
                        // KMK -->
                    }
                    // KMK <--
                }
            }
        }
    }

    private companion object {
        const val TURN_EPSILON = 0.004f
        const val TURN_DEGREES_X = 50f
        const val TURN_DEGREES_Y = 18f
        const val TURN_FADE = 0.5f
    }

    private data class Data(
        val transition: ChapterTransition,
        val currChapterDownloaded: Boolean,
        val goingToChapterDownloaded: Boolean,
    )
}
