package eu.kanade.tachiyomi.ui.browse.migration.sources

import androidx.compose.runtime.Immutable
import cafe.adriel.voyager.core.model.StateScreenModel
import cafe.adriel.voyager.core.model.screenModelScope
import eu.kanade.domain.source.interactor.GetSourcesWithFavoriteCount
import eu.kanade.domain.source.interactor.SetMigrateSorting
import eu.kanade.domain.source.model.installedExtension
import eu.kanade.domain.source.service.SourcePreferences
import eu.kanade.presentation.components.SEARCH_DEBOUNCE_MILLIS
import eu.kanade.tachiyomi.source.rust.RustSource
import eu.kanade.tachiyomi.source.rust.RustSources
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.persistentListOf
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.update
import logcat.LogPriority
import tachiyomi.core.common.preference.getAndSet
import tachiyomi.core.common.util.lang.launchIO
import tachiyomi.core.common.util.lang.withUIContext
import tachiyomi.core.common.util.system.logcat
import tachiyomi.data.DatabaseHandler
import tachiyomi.domain.manga.interactor.GetFavorites
import tachiyomi.domain.manga.model.Manga
import tachiyomi.domain.manga.model.MangaUpdate
import tachiyomi.domain.manga.repository.MangaRepository
import tachiyomi.domain.source.model.Source
import uy.kohesive.injekt.Injekt
import uy.kohesive.injekt.api.get

class MigrateSourceScreenModel(
    preferences: SourcePreferences = Injekt.get(),
    private val getSourcesWithFavoriteCount: GetSourcesWithFavoriteCount = Injekt.get(),
    private val setMigrateSorting: SetMigrateSorting = Injekt.get(),
    // KMK -->
    private val sourcePreferences: SourcePreferences = preferences,
    private val handler: DatabaseHandler = Injekt.get(),
    private val getFavorites: GetFavorites = Injekt.get(),
    private val mangaRepository: MangaRepository = Injekt.get(),
    // KMK <--
) : StateScreenModel<MigrateSourceScreenModel.State>(State()) {

    private val _channel = Channel<Event>(Int.MAX_VALUE)
    val channel = _channel.receiveAsFlow()

    init {
        screenModelScope.launchIO {
            // KMK -->
            combine(
                state.map { it.searchQuery }.distinctUntilChanged().debounce(SEARCH_DEBOUNCE_MILLIS),
                // KMK <--
                getSourcesWithFavoriteCount.subscribe(),
                // KMK -->
                RustSources.sources,
            ) { searchQuery, sourceCounts, rustSources ->
                val queryFilter: (String?) -> ((Pair<Source, Long>) -> Boolean) = { query ->
                    filter@{ pair ->
                        val source = pair.first
                        if (query.isNullOrBlank()) return@filter true
                        query.split(",").any {
                            val input = it.trim()
                            if (input.isEmpty()) return@any false
                            source.installedExtension?.name?.contains(input, ignoreCase = true) == true ||
                                source.name.contains(input, ignoreCase = true) ||
                                source.id == input.toLongOrNull()
                        }
                    }
                }
                sourceCounts.filter(queryFilter(searchQuery)) to rustMoves(sourceCounts, rustSources)
            }
                // KMK <--
                .catch {
                    logcat(LogPriority.ERROR, it)
                    _channel.send(Event.FailedFetchingSourcesWithCount)
                }
                .collectLatest { (sources, moves) ->
                    mutableState.update {
                        it.copy(
                            isLoading = false,
                            items = sources.toImmutableList(),
                            // KMK -->
                            rustMoves = moves.toImmutableList(),
                            // KMK <--
                        )
                    }
                }
        }

        preferences.migrationSortingDirection().changes()
            .onEach { mutableState.update { state -> state.copy(sortingDirection = it) } }
            .launchIn(screenModelScope)

        preferences.migrationSortingMode().changes()
            .onEach { mutableState.update { state -> state.copy(sortingMode = it) } }
            .launchIn(screenModelScope)
    }

    fun toggleSortingMode() {
        with(state.value) {
            val newMode = when (sortingMode) {
                SetMigrateSorting.Mode.ALPHABETICAL -> SetMigrateSorting.Mode.TOTAL
                SetMigrateSorting.Mode.TOTAL -> SetMigrateSorting.Mode.ALPHABETICAL
            }

            setMigrateSorting.await(newMode, sortingDirection)
        }
    }

    fun toggleSortingDirection() {
        with(state.value) {
            val newDirection = when (sortingDirection) {
                SetMigrateSorting.Direction.ASCENDING -> SetMigrateSorting.Direction.DESCENDING
                SetMigrateSorting.Direction.DESCENDING -> SetMigrateSorting.Direction.ASCENDING
            }

            setMigrateSorting.await(sortingMode, newDirection)
        }
    }

    // KMK -->
    /**
     * Library titles whose source is gone (an extension that was removed) and that have a Rust source of the same name
     * and language. Rust ports keep the extension's addresses, so the titles can move over as they are.
     */
    private fun rustMoves(counts: List<Pair<Source, Long>>, rust: List<RustSource>): List<RustMove> =
        counts.filter { (source, count) -> source.isStub && count > 0 }.mapNotNull { (source, count) ->
            rust.firstOrNull { it.lang == source.lang && (it.name.equals(source.name, true) || it.name.equals("${source.name} (Rust)", true)) }
                ?.let { RustMove(source, count, it) }
        }

    /** Kept here, not in the tab: the Browse screen recomposes when extension badges arrive and would forget it. */
    fun showRustMove(show: Boolean) = mutableState.update { it.copy(showRustMove = show) }

    /** The library titles of each source that can move, for the choice dialog. */
    suspend fun loadMoveTitles() {
        val sources = state.value.rustMoves.map { it.from.id }.toSet()
        val titles = getFavorites.await().filter { it.source in sources }.sortedBy { it.title.lowercase() }.groupBy { it.source }
        mutableState.update { it.copy(moveTitles = titles) }
    }

    /**
     * Points the chosen titles at the Rust sources: chapters, history and categories follow, since they hang on the title.
     * A source whose titles all go also takes its pin, saved searches and feeds along.
     */
    fun moveToRust(chosen: Set<Long>, onDone: (Int) -> Unit) {
        screenModelScope.launchIO { move(chosen).also { withUIContext { onDone(it) } } }
    }

    private suspend fun move(chosen: Set<Long>): Int {
        var moved = 0
        for (move in state.value.rustMoves) {
            val (from, to) = move.from.id to move.to.id
            val titles = state.value.moveTitles[from].orEmpty()
            val picked = titles.filter { it.id in chosen }
            if (picked.isEmpty()) continue
            runCatching {
                if (picked.size == titles.size) {
                    handler.await(inTransaction = true) {
                        ehQueries.migrateSource(to, from)
                        ehQueries.migrateMergedSource(to, from)
                        ehQueries.migrateSourceSavedSearch(to, from)
                        ehQueries.migrateSourceFeed(to, from)
                    }
                    sourcePreferences.pinnedSources().getAndSet { pinned -> if (from.toString() in pinned) pinned - from.toString() + to.toString() else pinned }
                } else {
                    mangaRepository.updateAll(picked.map { MangaUpdate(id = it.id, source = to) })
                }
                moved += picked.size
            }.onFailure { logcat(LogPriority.ERROR, it) { "Moving $from to the Rust source $to" } }
        }
        mutableState.update { it.copy(moveTitles = emptyMap()) }
        return moved
    }

    fun search(query: String?) {
        mutableState.update {
            it.copy(searchQuery = query)
        }
    }
    // KMK <--

    @Immutable
    data class State(
        val isLoading: Boolean = true,
        val items: ImmutableList<Pair<Source, Long>> = persistentListOf(),
        val sortingMode: SetMigrateSorting.Mode = SetMigrateSorting.Mode.ALPHABETICAL,
        val sortingDirection: SetMigrateSorting.Direction = SetMigrateSorting.Direction.ASCENDING,
        // KMK -->
        val searchQuery: String? = null,
        val rustMoves: ImmutableList<RustMove> = persistentListOf(),
        val moveTitles: Map<Long, List<Manga>> = emptyMap(),
        val showRustMove: Boolean = false,
        // KMK <--
    ) {
        val isEmpty = items.isEmpty()
    }

    sealed interface Event {
        data object FailedFetchingSourcesWithCount : Event
    }
}

// KMK -->
data class RustMove(val from: Source, val count: Long, val to: RustSource)
// KMK <--
