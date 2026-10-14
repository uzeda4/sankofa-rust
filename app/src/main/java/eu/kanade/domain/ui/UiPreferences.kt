package eu.kanade.domain.ui

import androidx.compose.material3.FabPosition
import com.materialkolor.PaletteStyle
import eu.kanade.domain.ui.model.AppTheme
import eu.kanade.domain.ui.model.TabletUiMode
import eu.kanade.domain.ui.model.ThemeMode
import tachiyomi.core.common.preference.PreferenceStore
import tachiyomi.core.common.preference.getEnum
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

class UiPreferences(
    private val preferenceStore: PreferenceStore,
) {

    // KMK -->
    // The redesign is dark-first; users can still pick light or follow the system.
    fun themeMode() = preferenceStore.getEnum("pref_theme_mode_key", ThemeMode.DARK)
    // KMK <--

    fun appTheme() = preferenceStore.getEnum(
        "pref_app_theme",
        // KMK -->
        AppTheme.REDESIGN,
        // KMK <--
    )

    fun themeDarkAmoled() = preferenceStore.getBoolean("pref_theme_dark_amoled_key", false)

    // KMK -->
    /** Turns off 3D tilt, parallax and blur everywhere, on top of what the system already asks for. */
    fun reduceMotion() = preferenceStore.getBoolean("pref_reduce_motion_key", false)

    /** Strength of the cover tilt, 0 to 100 percent. */
    fun tiltIntensity() = preferenceStore.getInt("pref_tilt_intensity_key", 100)
    // KMK <--

    // KMK -->
    fun colorTheme() = preferenceStore.getInt("pref_color_theme", 0xFFDF0090.toInt())

    fun customThemeStyle() = preferenceStore.getEnum("pref_custom_theme_style_key", PaletteStyle.Fidelity)

    fun themeCoverBased() = preferenceStore.getBoolean("pref_theme_cover_based_key", true)

    fun themeCoverBasedStyle() = preferenceStore.getEnum("pref_theme_cover_based_style_key", PaletteStyle.Vibrant)

    fun preloadLibraryColor() = preferenceStore.getBoolean("pref_preload_library_color_key", true)
    // KMK <--

    fun relativeTime() = preferenceStore.getBoolean("relative_time_v2", true)

    fun dateFormat() = preferenceStore.getString("app_date_format", "")

    fun tabletUiMode() = preferenceStore.getEnum("tablet_ui_mode", TabletUiMode.AUTOMATIC)

    fun imagesInDescription() = preferenceStore.getBoolean("pref_render_images_description", true)

    // SY -->

    fun expandFilters() = preferenceStore.getBoolean("eh_expand_filters", false)

    fun hideFeedTab() = preferenceStore.getBoolean("hide_latest_tab", false)

    fun feedTabInFront() = preferenceStore.getBoolean("latest_tab_position", false)

    // KMK -->
    fun expandRelatedMangas() = preferenceStore.getBoolean("expand_related_mangas", true)

    fun relatedMangasInOverflow() = preferenceStore.getBoolean("related_mangas_in_overflow", false)

    fun showHomeOnRelatedMangas() = preferenceStore.getBoolean("show_home_on_related_mangas", true)

    fun readButtonPosition() = preferenceStore.getString("reading_button_position", FabPosition.End.toString())

    fun usePanoramaCoverFlow() = preferenceStore.getBoolean("use_panorama_cover_flow", false)

    fun usePanoramaCoverAlways() = preferenceStore.getBoolean("use_panorama_cover_grid", true)

    fun usePanoramaCoverMangaInfo() = preferenceStore.getBoolean("use_panorama_cover_manga_info", false)

    fun topAlignCover() = preferenceStore.getBoolean("top_align_cover", false)
    // KMK <--

    fun recommendsInOverflow() = preferenceStore.getBoolean("recommends_in_overflow", false)

    fun mergeInOverflow() = preferenceStore.getBoolean("merge_in_overflow", true)

    fun previewsRowCount() = preferenceStore.getInt("pref_previews_row_count", 4)

    fun useNewSourceNavigation() = preferenceStore.getBoolean("use_new_source_navigation", true)

    fun bottomBarLabels() = preferenceStore.getBoolean("pref_show_bottom_bar_labels", true)

    fun showNavUpdates() = preferenceStore.getBoolean("pref_show_updates_button", true)

    fun showNavHistory() = preferenceStore.getBoolean("pref_show_history_button", true)

    // SY <--

    companion object {
        fun dateFormat(format: String): DateTimeFormatter = when (format) {
            "" -> DateTimeFormatter.ofLocalizedDate(FormatStyle.SHORT)
            else -> DateTimeFormatter.ofPattern(format, Locale.getDefault())
        }
    }
}
