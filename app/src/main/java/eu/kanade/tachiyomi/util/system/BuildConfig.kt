@file:Suppress("UNUSED", "KotlinConstantConditions")

package eu.kanade.tachiyomi.util.system

import eu.kanade.tachiyomi.BuildConfig

val telemetryIncluded: Boolean
    inline get() = BuildConfig.TELEMETRY_INCLUDED

// KMK -->
/** True when this build was told which repository publishes its releases (see UPDATE_REPO in app/build.gradle.kts). */
val updateRepoConfigured: Boolean
    inline get() = BuildConfig.UPDATE_REPO.isNotBlank()

val updaterEnabled: Boolean
    inline get() = BuildConfig.UPDATER_ENABLED && updateRepoConfigured
// KMK <--

val isDebugBuildType: Boolean
    inline get() = BuildConfig.BUILD_TYPE == "debug"

val isPreviewBuildType: Boolean
    inline get() = BuildConfig.BUILD_TYPE == "preview"

val isReleaseBuildType: Boolean
    inline get() = BuildConfig.BUILD_TYPE == "release"

val isFossBuildType: Boolean
    inline get() = BuildConfig.BUILD_TYPE == "foss"
