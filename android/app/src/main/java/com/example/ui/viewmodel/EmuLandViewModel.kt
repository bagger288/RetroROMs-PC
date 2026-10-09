package com.example.ui.viewmodel

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.example.data.downloader.RomDownloader
import com.example.data.local.AppDatabase
import com.example.data.remote.EmuLandScraper
import com.example.data.repository.EmuLandRepository
import com.example.model.CatalogCategory
import com.example.model.ConsoleInfo
import com.example.model.DownloadExecutionResult
import com.example.model.DownloadRecord
import com.example.model.GameCard
import com.example.model.RomFileVersion
import com.example.model.ZipExtractionRequest
import androidx.compose.ui.graphics.Color
import com.example.ui.screens.ColorTarget
import com.example.ui.theme.AppThemeColors
import com.example.ui.theme.CatalogViewMode
import com.example.ui.theme.DefaultDarkTheme
import com.example.ui.theme.DefaultLightTheme
import com.example.ui.theme.ThemeMode
import com.example.ui.theme.ThemePreferences
import com.example.ui.theme.ThemePreset
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

data class DownloadProgressState(
    val gameId: String,
    val percent: Int,
    val downloadedBytes: Long,
    val totalBytes: Long
)

enum class GameSort {
    RATING,
    TITLE,
    YEAR
}

class EmuLandViewModel(application: Application) : AndroidViewModel(application) {

    private val repository: EmuLandRepository

    init {
        val database = AppDatabase.getDatabase(application)
        val scraper = EmuLandScraper()
        val downloader = RomDownloader(application, database.downloadDao(), database.gameDao(), scraper)
        repository = EmuLandRepository(database, scraper, downloader)
    }

    // Navigation & UI tabs: 0 = Catalog, 1 = Downloads, 2 = Favorites
    private val _currentTab = MutableStateFlow(0)
    val currentTab: StateFlow<Int> = _currentTab.asStateFlow()

    // Consoles state
    val enabledConsoles: StateFlow<List<ConsoleInfo>> = repository.getEnabledConsoles()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), emptyList())

    val allConsoles: StateFlow<List<ConsoleInfo>> = repository.getAllConsoles()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), emptyList())

    private val _selectedConsole = MutableStateFlow<ConsoleInfo?>(null)
    val selectedConsole: StateFlow<ConsoleInfo?> = _selectedConsole.asStateFlow()

    // Available categories dynamically derived from console or pagelist_top
    private val _availableCategories = MutableStateFlow<List<CatalogCategory>>(emptyList())
    val availableCategories: StateFlow<List<CatalogCategory>> = _availableCategories.asStateFlow()

    // Sub-category: "top" (Popular), "best" (Best Rated), or alphabetical ("0-9", "a", "b", etc.)
    private val _selectedCategory = MutableStateFlow("top")
    val selectedCategory: StateFlow<String> = _selectedCategory.asStateFlow()

    private val _currentPage = MutableStateFlow(1)
    val currentPage: StateFlow<Int> = _currentPage.asStateFlow()

    private val _totalPages = MutableStateFlow(1)
    val totalPages: StateFlow<Int> = _totalPages.asStateFlow()

    private val _hasNextPage = MutableStateFlow(false)
    val hasNextPage: StateFlow<Boolean> = _hasNextPage.asStateFlow()

    private val _hasPrevPage = MutableStateFlow(false)
    val hasPrevPage: StateFlow<Boolean> = _hasPrevPage.asStateFlow()

    private val _currentCatalogGames = MutableStateFlow<List<GameCard>>(emptyList())
    private var catalogLoadJob: Job? = null

    // Search and Sort
    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    private val _selectedSort = MutableStateFlow(GameSort.RATING)
    val selectedSort: StateFlow<GameSort> = _selectedSort.asStateFlow()

    private var searchJob: Job? = null
    private val _searchResults = MutableStateFlow<List<GameCard>>(emptyList())
    val searchResults: StateFlow<List<GameCard>> = _searchResults.asStateFlow()

    private val _isSearching = MutableStateFlow(false)
    val isSearching: StateFlow<Boolean> = _isSearching.asStateFlow()

    private val _searchPlatformFilter = MutableStateFlow<String?>(null)
    val searchPlatformFilter: StateFlow<String?> = _searchPlatformFilter.asStateFlow()

    private val _searchCriteria = combine(_searchQuery, _searchResults, _searchPlatformFilter) { query, results, platform ->
        Triple(query, results, platform)
    }

    // Games state
    val displayedGames: StateFlow<List<GameCard>> = combine(
        _currentCatalogGames,
        _searchCriteria,
        _selectedSort,
        repository.getFavorites()
    ) { catalogList, searchTuple, sort, favorites ->
        val query = searchTuple.first
        val searchList = searchTuple.second
        val platformFilter = searchTuple.third
        val favIds = favorites.map { it.id }.toSet()
        val explicitList = if (query.isNotBlank()) {
            val filtered = if (platformFilter != null) {
                searchList.filter { it.consoleSlug.equals(platformFilter, ignoreCase = true) }
            } else {
                searchList
            }
            filtered.map { it.copy(isFavorite = it.id in favIds) }
        } else {
            catalogList.map { it.copy(isFavorite = it.id in favIds) }
        }

        when (sort) {
            GameSort.RATING -> explicitList.sortedByDescending { it.rating }
            GameSort.TITLE -> explicitList.sortedBy { it.title }
            GameSort.YEAR -> explicitList.sortedByDescending { it.year }
        }
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), emptyList())

    // Favorites
    val favoriteGames: StateFlow<List<GameCard>> = repository.getFavorites()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), emptyList())

    // Downloads
    val downloadRecords: StateFlow<List<DownloadRecord>> = repository.getAllDownloads()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), emptyList())

    // Active downloading states
    private val _activeDownloads = MutableStateFlow<Map<String, DownloadProgressState>>(emptyMap())
    val activeDownloads: StateFlow<Map<String, DownloadProgressState>> = _activeDownloads.asStateFlow()

    // Selected game for detail modal bottom sheet
    private val _selectedGameForDetail = MutableStateFlow<GameCard?>(null)
    val selectedGameForDetail: StateFlow<GameCard?> = _selectedGameForDetail.asStateFlow()

    private val _isLoadingDetail = MutableStateFlow(false)
    val isLoadingDetail: StateFlow<Boolean> = _isLoadingDetail.asStateFlow()

    // Manage consoles dialog visibility
    private val _showManageConsoles = MutableStateFlow(false)
    val showManageConsoles: StateFlow<Boolean> = _showManageConsoles.asStateFlow()

    // Refreshing state
    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing.asStateFlow()

    // Selected game for ROM versions sheet
    private val _selectedGameForVersions = MutableStateFlow<GameCard?>(null)
    val selectedGameForVersions: StateFlow<GameCard?> = _selectedGameForVersions.asStateFlow()

    private val _availableVersions = MutableStateFlow<List<RomFileVersion>>(emptyList())
    val availableVersions: StateFlow<List<RomFileVersion>> = _availableVersions.asStateFlow()

    private val _isLoadingVersions = MutableStateFlow(false)
    val isLoadingVersions: StateFlow<Boolean> = _isLoadingVersions.asStateFlow()

    // Theme & Color Customization
    private val themePrefs = ThemePreferences(application)

    private val _themeMode = MutableStateFlow(themePrefs.getThemeMode())
    val themeMode: StateFlow<ThemeMode> = _themeMode.asStateFlow()

    private val _customPrimary = MutableStateFlow(themePrefs.getCustomPrimary())
    val customPrimary: StateFlow<Color> = _customPrimary.asStateFlow()

    private val _customBackground = MutableStateFlow(themePrefs.getCustomBackground())
    val customBackground: StateFlow<Color> = _customBackground.asStateFlow()

    private val _customSurface = MutableStateFlow(themePrefs.getCustomSurface())
    val customSurface: StateFlow<Color> = _customSurface.asStateFlow()

    private val _customSecondary = MutableStateFlow(themePrefs.getCustomSecondary())
    val customSecondary: StateFlow<Color> = _customSecondary.asStateFlow()

    private val _customOnPrimary = MutableStateFlow(themePrefs.getCustomOnPrimary())
    val customOnPrimary: StateFlow<Color> = _customOnPrimary.asStateFlow()

    val currentAppTheme: StateFlow<AppThemeColors> = combine(
        _themeMode,
        _customPrimary,
        _customBackground,
        _customSurface,
        _customSecondary,
        _customOnPrimary
    ) { args: Array<Any?> ->
        val mode = args[0] as ThemeMode
        val primary = args[1] as Color
        val bg = args[2] as Color
        val surface = args[3] as Color
        val secondary = args[4] as Color
        val onPrimary = args[5] as Color
        when (mode) {
            ThemeMode.DARK -> DefaultDarkTheme
            ThemeMode.LIGHT -> DefaultLightTheme
            ThemeMode.CUSTOM -> {
                val isLight = (0.299f * bg.red + 0.587f * bg.green + 0.114f * bg.blue) > 0.5f
                val variant = if (isLight) Color(0xFFEAEFF5) else surface.copy(alpha = 0.9f)
                val container = if (isLight) Color(0xFFDFE6F0) else Color(0xFF232B3E)
                val border = if (isLight) Color(0xFFCBD5E1) else Color(0xFF2F3B52)
                val textP = if (isLight) Color(0xFF0F172A) else Color(0xFFF1F5F9)
                val textS = if (isLight) Color(0xFF334155) else Color(0xFF94A3B8)
                val textM = if (isLight) Color(0xFF64748B) else Color(0xFF64748B)

                AppThemeColors(
                    primaryAccent = primary,
                    secondaryAccent = secondary,
                    tertiaryAccent = Color(0xFFFF3366),
                    background = bg,
                    surface = surface,
                    surfaceVariant = variant,
                    surfaceContainer = container,
                    cardBorder = border,
                    textPrimary = textP,
                    textSecondary = textS,
                    textMuted = textM,
                    onPrimaryAccent = onPrimary,
                    isLight = isLight
                )
            }
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, DefaultDarkTheme)

    // Catalog View Mode (GRID or LIST)
    private val _catalogViewMode = MutableStateFlow(themePrefs.getCatalogViewMode())
    val catalogViewMode: StateFlow<CatalogViewMode> = _catalogViewMode.asStateFlow()

    fun setCatalogViewMode(mode: CatalogViewMode) {
        _catalogViewMode.value = mode
        themePrefs.setCatalogViewMode(mode)
    }

    fun toggleCatalogViewMode() {
        val next = if (_catalogViewMode.value == CatalogViewMode.GRID) CatalogViewMode.LIST else CatalogViewMode.GRID
        setCatalogViewMode(next)
    }

    // Custom SAF download folder
    private val _customFolderUri = MutableStateFlow<String?>(repository.getCustomFolderUri())
    val customFolderUri: StateFlow<String?> = _customFolderUri.asStateFlow()

    // Zip extraction state
    private val _pendingZipExtraction = MutableStateFlow<ZipExtractionRequest?>(null)
    val pendingZipExtraction: StateFlow<ZipExtractionRequest?> = _pendingZipExtraction.asStateFlow()

    private val _isAutoUnpackEnabled = MutableStateFlow(repository.isAutoUnpackEnabled())
    val isAutoUnpackEnabled: StateFlow<Boolean> = _isAutoUnpackEnabled.asStateFlow()

    private val _isDeleteZipAfterUnpack = MutableStateFlow(repository.isDeleteZipAfterUnpack())
    val isDeleteZipAfterUnpack: StateFlow<Boolean> = _isDeleteZipAfterUnpack.asStateFlow()

    // Snackbars / Feedback messages
    private val _userMessage = MutableSharedFlow<String>()
    val userMessage = _userMessage.asSharedFlow()

    fun selectTab(tab: Int) {
        _currentTab.value = tab
    }

    fun selectConsole(console: ConsoleInfo) {
        _selectedConsole.value = console
        _searchQuery.value = ""
        val initialCats = repository.getInitialCategoriesForConsole(console.slug)
        _availableCategories.value = initialCats
        val targetCat = when {
            initialCats.any { it.key.equals("top", ignoreCase = true) } -> "top"
            initialCats.any { it.key.equals("best", ignoreCase = true) } -> "best"
            initialCats.isNotEmpty() -> initialCats.first().key
            else -> "all"
        }
        _selectedCategory.value = targetCat
        _currentPage.value = 1
        loadCatalogGames(console, targetCat, 1, append = false)
    }

    fun selectCategory(category: String) {
        _selectedCategory.value = category
        _currentPage.value = 1
        val console = _selectedConsole.value ?: enabledConsoles.value.firstOrNull() ?: return
        loadCatalogGames(console, category, 1, append = false)
    }

    fun goToPage(page: Int) {
        val targetPage = page.coerceAtLeast(1)
        _currentPage.value = targetPage
        val console = _selectedConsole.value ?: enabledConsoles.value.firstOrNull() ?: return
        loadCatalogGames(console, _selectedCategory.value, targetPage, append = false)
    }

    fun goToNextPage() {
        if (_hasNextPage.value || _currentPage.value < _totalPages.value) {
            goToPage(_currentPage.value + 1)
        }
    }

    fun goToPrevPage() {
        if (_currentPage.value > 1) {
            goToPage(_currentPage.value - 1)
        }
    }

    fun loadNextPageAppend() {
        val nextPage = _currentPage.value + 1
        val console = _selectedConsole.value ?: enabledConsoles.value.firstOrNull() ?: return
        loadCatalogGames(console, _selectedCategory.value, nextPage, append = true)
    }

    fun loadNextPage() {
        goToNextPage()
    }

    private fun loadCatalogGames(console: ConsoleInfo, category: String, page: Int, append: Boolean) {
        catalogLoadJob?.cancel()
        catalogLoadJob = viewModelScope.launch {
            _isRefreshing.value = true
            if (!append) {
                _currentCatalogGames.value = emptyList()
            }
            try {
                val result = repository.fetchGamesPageForConsole(
                    slug = console.slug,
                    consoleName = console.name,
                    section = console.section,
                    category = category,
                    page = page
                )
                result.onSuccess { pageResult ->
                    _currentPage.value = pageResult.currentPage
                    _totalPages.value = pageResult.totalPages.coerceAtLeast(1)
                    _hasNextPage.value = pageResult.hasNextPage || (pageResult.currentPage < pageResult.totalPages)
                    _hasPrevPage.value = pageResult.currentPage > 1

                    if (pageResult.availableCategories.isNotEmpty()) {
                        _availableCategories.value = pageResult.availableCategories
                    }

                    if (append) {
                        _currentCatalogGames.value = (_currentCatalogGames.value + pageResult.games).distinctBy { it.id }
                    } else {
                        _currentCatalogGames.value = pageResult.games
                    }
                }.onFailure {
                    _userMessage.emit("Не удалось загрузить страницу $page с Emu-Land")
                }
            } catch (_: Exception) {
            } finally {
                _isRefreshing.value = false
            }
        }
    }

    fun setSearchPlatformFilter(slug: String?) {
        _searchPlatformFilter.value = slug
    }

    fun setSearchQuery(query: String) {
        _searchQuery.value = query
        searchJob?.cancel()

        if (query.isBlank()) {
            _searchResults.value = emptyList()
            _isSearching.value = false
            _searchPlatformFilter.value = null
            return
        }

        searchJob = viewModelScope.launch {
            delay(350)
            _isSearching.value = true
            try {
                val results = repository.searchGamesSiteWide(query)
                _searchResults.value = results
                if (results.isNotEmpty()) {
                    _userMessage.emit("Найдено ${results.size} игр по запросу «$query»")
                }
            } catch (e: Exception) {
                _searchResults.value = emptyList()
            } finally {
                _isSearching.value = false
            }
        }
    }

    fun setSort(sort: GameSort) {
        _selectedSort.value = sort
    }

    fun openGameDetail(game: GameCard) {
        _selectedGameForDetail.value = game
        viewModelScope.launch {
            _isLoadingDetail.value = true
            try {
                val fullGame = repository.loadFullGameDetails(game)
                _selectedGameForDetail.value = fullGame
            } catch (_: Exception) {
            } finally {
                _isLoadingDetail.value = false
            }
        }
    }

    fun closeGameDetail() {
        _selectedGameForDetail.value = null
    }

    fun toggleFavorite(game: GameCard) {
        viewModelScope.launch {
            repository.toggleFavorite(game.id, game.isFavorite)
        }
    }

    fun toggleConsoleEnabled(slug: String, isEnabled: Boolean) {
        viewModelScope.launch {
            repository.setConsoleEnabled(slug, isEnabled)
        }
    }

    fun setAllConsolesEnabled(isEnabled: Boolean) {
        viewModelScope.launch {
            repository.setAllConsolesEnabled(isEnabled)
        }
    }

    fun moveConsoleUp(slug: String) {
        viewModelScope.launch {
            repository.moveConsoleUp(slug)
        }
    }

    fun moveConsoleDown(slug: String) {
        viewModelScope.launch {
            repository.moveConsoleDown(slug)
        }
    }

    fun moveConsoleToTop(slug: String) {
        viewModelScope.launch {
            repository.moveConsoleToTop(slug)
        }
    }

    fun resetConsolesOrder() {
        viewModelScope.launch {
            repository.resetConsolesOrder()
            _userMessage.emit("Порядок платформ сброшен по умолчанию")
        }
    }

    fun setShowManageConsoles(show: Boolean) {
        _showManageConsoles.value = show
    }

    fun setCustomDownloadFolder(uriString: String?) {
        repository.setCustomFolderUri(uriString)
        _customFolderUri.value = uriString
    }

    fun requestDownloadGame(game: GameCard) {
        _selectedGameForDetail.value = null
        _selectedGameForVersions.value = game
        viewModelScope.launch {
            _isLoadingVersions.value = true
            try {
                val versions = repository.getRomVersions(game)
                _availableVersions.value = versions
            } catch (e: Exception) {
                Log.e("EmuLandViewModel", "Failed to load versions for ${game.title}: ${e.message}", e)
                _availableVersions.value = emptyList()
            } finally {
                _isLoadingVersions.value = false
            }
        }
    }

    fun closeVersionsSheet() {
        _selectedGameForVersions.value = null
        _availableVersions.value = emptyList()
    }

    fun downloadSelectedVersion(game: GameCard, version: RomFileVersion) {
        closeVersionsSheet()
        val consoleFolder = enabledConsoles.value.find { it.slug == game.consoleSlug }?.folderName
            ?: game.consoleName.replace("/", "_")

        viewModelScope.launch {
            _userMessage.emit("Загрузка: ${version.name}…")

            _activeDownloads.value = _activeDownloads.value + (game.id to DownloadProgressState(game.id, 0, 0, 0))

            val result = repository.downloadSpecificVersion(game, version, consoleFolder) { percent, downloaded, total ->
                _activeDownloads.value = _activeDownloads.value + (game.id to DownloadProgressState(game.id, percent, downloaded, total))
            }

            _activeDownloads.value = _activeDownloads.value - game.id

            when (result) {
                is DownloadExecutionResult.Completed -> {
                    _userMessage.emit(result.message)
                }
                is DownloadExecutionResult.RequiresSelection -> {
                    _pendingZipExtraction.value = result.request
                }
                is DownloadExecutionResult.Failed -> {
                    _userMessage.emit("Ошибка загрузки: ${result.error.message ?: "сбой сети"}")
                }
            }
        }
    }

    fun confirmZipExtraction(request: ZipExtractionRequest, selectedEntryNames: List<String>) {
        viewModelScope.launch {
            _pendingZipExtraction.value = null
            _userMessage.emit("Извлечение ${selectedEntryNames.size} файлов…")
            val result = repository.extractSelectedZipEntries(request, selectedEntryNames)
            result.onSuccess { paths ->
                val folderDisplay = paths.firstOrNull()?.substringBeforeLast("/")?.substringAfterLast("/") ?: request.consoleFolderName
                _userMessage.emit("Извлечено ${paths.size} файлов в $folderDisplay")
            }.onFailure { err ->
                _userMessage.emit("Ошибка распаковки: ${err.message ?: "сбой"}")
            }
        }
    }

    fun keepZipWithoutExtraction(request: ZipExtractionRequest) {
        viewModelScope.launch {
            _pendingZipExtraction.value = null
            val result = repository.keepZipWithoutExtraction(request)
            result.onSuccess { path ->
                _userMessage.emit("Сохранен ZIP-архив: ${path.substringAfterLast("/")}")
            }.onFailure { err ->
                _userMessage.emit("Ошибка сохранения: ${err.message ?: "сбой"}")
            }
        }
    }

    fun dismissZipExtraction(request: ZipExtractionRequest) {
        _pendingZipExtraction.value = null
        viewModelScope.launch {
            repository.dismissZipExtraction(request)
        }
    }

    fun setAutoUnpackEnabled(enabled: Boolean) {
        _isAutoUnpackEnabled.value = enabled
        repository.setAutoUnpackEnabled(enabled)
    }

    fun setDeleteZipAfterUnpack(enabled: Boolean) {
        _isDeleteZipAfterUnpack.value = enabled
        repository.setDeleteZipAfterUnpack(enabled)
    }

    fun downloadGame(game: GameCard) {
        // By default open versions sheet so user can choose specific release
        requestDownloadGame(game)
    }

    fun refreshFromWeb() {
        viewModelScope.launch {
            _isRefreshing.value = true
            try {
                repository.refreshConsolesFromWeb()
                val currentC = _selectedConsole.value ?: enabledConsoles.value.firstOrNull()
                if (currentC != null) {
                    loadCatalogGames(currentC, _selectedCategory.value, _currentPage.value, append = false)
                }
                _userMessage.emit("Каталог обновлен с Emu-Land")
            } catch (_: Exception) {
                _userMessage.emit("Каталог обновлен (из кэша)")
            } finally {
                _isRefreshing.value = false
            }
        }
    }

    fun deleteDownload(id: Long) {
        viewModelScope.launch {
            repository.deleteDownload(id)
        }
    }

    fun setThemeMode(mode: ThemeMode) {
        _themeMode.value = mode
        themePrefs.setThemeMode(mode)
    }

    fun applyPreset(preset: ThemePreset) {
        _themeMode.value = ThemeMode.CUSTOM
        themePrefs.setThemeMode(ThemeMode.CUSTOM)

        _customPrimary.value = preset.primary
        themePrefs.setCustomPrimary(preset.primary)

        _customBackground.value = preset.background
        themePrefs.setCustomBackground(preset.background)

        _customSurface.value = preset.surface
        themePrefs.setCustomSurface(preset.surface)

        _customSecondary.value = preset.secondary
        themePrefs.setCustomSecondary(preset.secondary)

        _customOnPrimary.value = preset.onPrimary
        themePrefs.setCustomOnPrimary(preset.onPrimary)
    }

    fun updateCustomColor(target: ColorTarget, color: Color) {
        _themeMode.value = ThemeMode.CUSTOM
        themePrefs.setThemeMode(ThemeMode.CUSTOM)
        when (target) {
            ColorTarget.PRIMARY -> {
                _customPrimary.value = color
                themePrefs.setCustomPrimary(color)
            }
            ColorTarget.SECONDARY -> {
                _customSecondary.value = color
                themePrefs.setCustomSecondary(color)
            }
            ColorTarget.BACKGROUND -> {
                _customBackground.value = color
                themePrefs.setCustomBackground(color)
            }
            ColorTarget.SURFACE -> {
                _customSurface.value = color
                themePrefs.setCustomSurface(color)
            }
            ColorTarget.ON_PRIMARY -> {
                _customOnPrimary.value = color
                themePrefs.setCustomOnPrimary(color)
            }
        }
    }

    fun resetColorsToDefault() {
        themePrefs.resetToDefaults()
        _themeMode.value = ThemeMode.DARK
        _customPrimary.value = DefaultDarkTheme.primaryAccent
        _customBackground.value = DefaultDarkTheme.background
        _customSurface.value = DefaultDarkTheme.surface
        _customSecondary.value = DefaultDarkTheme.secondaryAccent
        _customOnPrimary.value = DefaultDarkTheme.onPrimaryAccent
    }
}
