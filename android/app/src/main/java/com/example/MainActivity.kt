package com.example

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.Gamepad
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material.icons.outlined.FavoriteBorder
import androidx.compose.material.icons.outlined.Gamepad
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.ui.screens.DownloadsScreen
import com.example.ui.screens.FavoritesScreen
import com.example.ui.screens.GameDetailSheet
import com.example.ui.screens.HomeScreen
import com.example.ui.screens.RomVersionsSheet
import com.example.ui.screens.SettingsDialog
import com.example.ui.screens.ZipExtractionDialog
import com.example.ui.theme.ArcadeCardBorder
import com.example.ui.theme.ArcadeDarkBg
import com.example.ui.theme.ArcadeSurface
import com.example.ui.theme.ArcadeSurfaceVariant
import com.example.ui.theme.MyApplicationTheme
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.OnPrimary
import com.example.ui.theme.TextMuted
import com.example.ui.theme.TextSecondary
import com.example.ui.viewmodel.EmuLandViewModel
import kotlinx.coroutines.flow.collectLatest

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            val viewModel: EmuLandViewModel = viewModel()
            val currentAppTheme by viewModel.currentAppTheme.collectAsStateWithLifecycle()
            MyApplicationTheme(appColors = currentAppTheme) {
                RetroROMsApp(viewModel)
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RetroROMsApp(viewModel: EmuLandViewModel = viewModel()) {
    val currentTab by viewModel.currentTab.collectAsStateWithLifecycle()
    val enabledConsoles by viewModel.enabledConsoles.collectAsStateWithLifecycle()
    val allConsoles by viewModel.allConsoles.collectAsStateWithLifecycle()
    val selectedConsole by viewModel.selectedConsole.collectAsStateWithLifecycle()
    val displayedGames by viewModel.displayedGames.collectAsStateWithLifecycle()
    val favoriteGames by viewModel.favoriteGames.collectAsStateWithLifecycle()
    val downloadRecords by viewModel.downloadRecords.collectAsStateWithLifecycle()
    val activeDownloads by viewModel.activeDownloads.collectAsStateWithLifecycle()
    val searchQuery by viewModel.searchQuery.collectAsStateWithLifecycle()
    val selectedSort by viewModel.selectedSort.collectAsStateWithLifecycle()
    val selectedCategory by viewModel.selectedCategory.collectAsStateWithLifecycle()
    val availableCategories by viewModel.availableCategories.collectAsStateWithLifecycle()
    val currentPage by viewModel.currentPage.collectAsStateWithLifecycle()
    val totalPages by viewModel.totalPages.collectAsStateWithLifecycle()
    val hasNextPage by viewModel.hasNextPage.collectAsStateWithLifecycle()
    val hasPrevPage by viewModel.hasPrevPage.collectAsStateWithLifecycle()
    val isRefreshing by viewModel.isRefreshing.collectAsStateWithLifecycle()
    val customFolderUri by viewModel.customFolderUri.collectAsStateWithLifecycle()

    val selectedGameForDetail by viewModel.selectedGameForDetail.collectAsStateWithLifecycle()
    val isLoadingDetail by viewModel.isLoadingDetail.collectAsStateWithLifecycle()
    val showManageConsoles by viewModel.showManageConsoles.collectAsStateWithLifecycle()

    val selectedGameForVersions by viewModel.selectedGameForVersions.collectAsStateWithLifecycle()
    val availableVersions by viewModel.availableVersions.collectAsStateWithLifecycle()
    val isLoadingVersions by viewModel.isLoadingVersions.collectAsStateWithLifecycle()
    val pendingZipExtraction by viewModel.pendingZipExtraction.collectAsStateWithLifecycle()
    val isAutoUnpackEnabled by viewModel.isAutoUnpackEnabled.collectAsStateWithLifecycle()
    val isDeleteZipAfterUnpack by viewModel.isDeleteZipAfterUnpack.collectAsStateWithLifecycle()

    val allSearchResults by viewModel.searchResults.collectAsStateWithLifecycle()
    val isSearching by viewModel.isSearching.collectAsStateWithLifecycle()
    val searchPlatformFilter by viewModel.searchPlatformFilter.collectAsStateWithLifecycle()

    val themeMode by viewModel.themeMode.collectAsStateWithLifecycle()
    val customPrimary by viewModel.customPrimary.collectAsStateWithLifecycle()
    val customOnPrimary by viewModel.customOnPrimary.collectAsStateWithLifecycle()
    val customBackground by viewModel.customBackground.collectAsStateWithLifecycle()
    val customSurface by viewModel.customSurface.collectAsStateWithLifecycle()
    val customSecondary by viewModel.customSecondary.collectAsStateWithLifecycle()
    val catalogViewMode by viewModel.catalogViewMode.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val versionsSheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    // Collect snackbar messages
    LaunchedEffect(Unit) {
        viewModel.userMessage.collectLatest { msg ->
            snackbarHostState.showSnackbar(msg)
        }
    }

    // Default select first console if none selected or if previously selected console was disabled
    LaunchedEffect(enabledConsoles) {
        if (enabledConsoles.isNotEmpty() && (selectedConsole == null || enabledConsoles.none { it.slug == selectedConsole?.slug })) {
            viewModel.selectConsole(enabledConsoles.first())
        }
    }

    // Back handling
    BackHandler(enabled = currentTab != 0 || selectedGameForDetail != null || selectedGameForVersions != null || showManageConsoles || pendingZipExtraction != null) {
        if (pendingZipExtraction != null) {
            viewModel.dismissZipExtraction(pendingZipExtraction!!)
        } else if (selectedGameForVersions != null) {
            viewModel.closeVersionsSheet()
        } else if (selectedGameForDetail != null) {
            viewModel.closeGameDetail()
        } else if (showManageConsoles) {
            viewModel.setShowManageConsoles(false)
        } else if (currentTab != 0) {
            viewModel.selectTab(0)
        }
    }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        containerColor = ArcadeDarkBg,
        contentWindowInsets = WindowInsets.safeDrawing,
        snackbarHost = { SnackbarHost(snackbarHostState) },
        bottomBar = {
            NavigationBar(
                containerColor = ArcadeSurface,
                tonalElevation = 4.dp,
                modifier = Modifier
                    .background(ArcadeSurface)
                    .windowInsetsPadding(WindowInsets.navigationBars)
                    .testTag("main_navigation_bar")
            ) {
                NavigationBarItem(
                    selected = currentTab == 0 && !showManageConsoles,
                    onClick = {
                        if (showManageConsoles) viewModel.setShowManageConsoles(false)
                        viewModel.selectTab(0)
                    },
                    icon = {
                        Icon(
                            painter = painterResource(
                                id = if (currentTab == 0 && !showManageConsoles) R.drawable.ic_game_cartridge else R.drawable.ic_game_cartridge_outlined
                            ),
                            contentDescription = "Каталог",
                            modifier = Modifier.size(22.dp)
                        )
                    },
                    label = { Text("Каталог", fontSize = 11.sp) },
                    colors = NavigationBarItemDefaults.colors(
                        selectedIconColor = OnPrimary,
                        selectedTextColor = NeonCyan,
                        indicatorColor = NeonCyan,
                        unselectedIconColor = TextMuted,
                        unselectedTextColor = TextMuted
                    ),
                    modifier = Modifier.testTag("nav_tab_catalog")
                )

                NavigationBarItem(
                    selected = currentTab == 1 && !showManageConsoles,
                    onClick = {
                        if (showManageConsoles) viewModel.setShowManageConsoles(false)
                        viewModel.selectTab(1)
                    },
                    icon = {
                        Icon(
                            imageVector = if (currentTab == 1 && !showManageConsoles) Icons.Default.Download else Icons.Outlined.Download,
                            contentDescription = "Загрузки"
                        )
                    },
                    label = {
                        val badge = if (activeDownloads.isNotEmpty()) " (${activeDownloads.size})" else ""
                        Text("Загрузки$badge", fontSize = 11.sp)
                    },
                    colors = NavigationBarItemDefaults.colors(
                        selectedIconColor = OnPrimary,
                        selectedTextColor = NeonCyan,
                        indicatorColor = NeonCyan,
                        unselectedIconColor = TextMuted,
                        unselectedTextColor = TextMuted
                    ),
                    modifier = Modifier.testTag("nav_tab_downloads")
                )

                NavigationBarItem(
                    selected = currentTab == 2 && !showManageConsoles,
                    onClick = {
                        if (showManageConsoles) viewModel.setShowManageConsoles(false)
                        viewModel.selectTab(2)
                    },
                    icon = {
                        Icon(
                            imageVector = if (currentTab == 2 && !showManageConsoles) Icons.Default.Favorite else Icons.Outlined.FavoriteBorder,
                            contentDescription = "Избранное"
                        )
                    },
                    label = { Text("Избранное", fontSize = 11.sp) },
                    colors = NavigationBarItemDefaults.colors(
                        selectedIconColor = OnPrimary,
                        selectedTextColor = NeonCyan,
                        indicatorColor = NeonCyan,
                        unselectedIconColor = TextMuted,
                        unselectedTextColor = TextMuted
                    ),
                    modifier = Modifier.testTag("nav_tab_favorites")
                )

                NavigationBarItem(
                    selected = showManageConsoles,
                    onClick = { viewModel.setShowManageConsoles(true) },
                    icon = {
                        Icon(
                            imageVector = if (showManageConsoles) Icons.Default.Settings else Icons.Outlined.Settings,
                            contentDescription = "Настройки"
                        )
                    },
                    label = { Text("Настройки", fontSize = 11.sp) },
                    colors = NavigationBarItemDefaults.colors(
                        selectedIconColor = OnPrimary,
                        selectedTextColor = NeonCyan,
                        indicatorColor = NeonCyan,
                        unselectedIconColor = TextMuted,
                        unselectedTextColor = TextMuted
                    ),
                    modifier = Modifier.testTag("nav_tab_settings")
                )
            }
        }
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
        ) {
            when (currentTab) {
                0 -> {
                    HomeScreen(
                        enabledConsoles = enabledConsoles,
                        selectedConsole = selectedConsole,
                        selectedCategory = selectedCategory,
                        categories = availableCategories,
                        currentPage = currentPage,
                        totalPages = totalPages,
                        hasNextPage = hasNextPage,
                        hasPrevPage = hasPrevPage,
                        games = displayedGames,
                        allSearchResults = allSearchResults,
                        isSearching = isSearching,
                        searchPlatformFilter = searchPlatformFilter,
                        activeDownloads = activeDownloads,
                        searchQuery = searchQuery,
                        selectedSort = selectedSort,
                        isRefreshing = isRefreshing,
                        onSelectConsole = { viewModel.selectConsole(it) },
                        onSelectCategory = { viewModel.selectCategory(it) },
                        onGoToPage = { viewModel.goToPage(it) },
                        onNextPage = { viewModel.goToNextPage() },
                        onPrevPage = { viewModel.goToPrevPage() },
                        onAppendNextPage = { viewModel.loadNextPageAppend() },
                        onLoadNextPage = { viewModel.goToNextPage() },
                        onSearchQueryChange = { viewModel.setSearchQuery(it) },
                        onSelectSearchPlatformFilter = { viewModel.setSearchPlatformFilter(it) },
                        onSelectSort = { viewModel.setSort(it) },
                        onCardClick = { viewModel.openGameDetail(it) },
                        onDownloadClick = { viewModel.downloadGame(it) },
                        onFavoriteClick = { viewModel.toggleFavorite(it) },
                        onOpenManageConsoles = { viewModel.setShowManageConsoles(true) },
                        onRefresh = { viewModel.refreshFromWeb() },
                        catalogViewMode = catalogViewMode,
                        onSelectCatalogViewMode = { viewModel.setCatalogViewMode(it) }
                    )
                }

                1 -> {
                    DownloadsScreen(
                        downloads = downloadRecords,
                        activeDownloads = activeDownloads,
                        customFolderUri = customFolderUri,
                        onSetCustomFolder = { viewModel.setCustomDownloadFolder(it) },
                        onDeleteDownload = { viewModel.deleteDownload(it) },
                        isAutoUnpackEnabled = isAutoUnpackEnabled,
                        onToggleAutoUnpack = { viewModel.setAutoUnpackEnabled(it) },
                        isDeleteZipAfterUnpack = isDeleteZipAfterUnpack,
                        onToggleDeleteZip = { viewModel.setDeleteZipAfterUnpack(it) }
                    )
                }

                2 -> {
                    FavoritesScreen(
                        favorites = favoriteGames,
                        activeDownloads = activeDownloads,
                        onCardClick = { viewModel.openGameDetail(it) },
                        onDownloadClick = { viewModel.downloadGame(it) },
                        onFavoriteClick = { viewModel.toggleFavorite(it) }
                    )
                }
            }
        }
    }

    // Modal Game Detail Bottom Sheet
    selectedGameForDetail?.let { game ->
        val folder = enabledConsoles.find { it.slug == game.consoleSlug }?.folderName
            ?: game.consoleName.replace("/", "_")

        GameDetailSheet(
            game = game,
            targetFolderName = folder,
            downloadProgress = activeDownloads[game.id],
            isLoadingDetail = isLoadingDetail,
            sheetState = sheetState,
            onDismiss = { viewModel.closeGameDetail() },
            onDownloadClick = { viewModel.downloadGame(game) }
        )
    }

    // Modal ROM Versions Bottom Sheet
    selectedGameForVersions?.let { game ->
        RomVersionsSheet(
            game = game,
            versions = availableVersions,
            isLoading = isLoadingVersions,
            sheetState = versionsSheetState,
            onSelectVersion = { g, version ->
                viewModel.downloadSelectedVersion(g, version)
            },
            onDismiss = { viewModel.closeVersionsSheet() }
        )
    }

    // Unified Settings & Customization Dialog (Platforms & Theme/Colors)
    if (showManageConsoles) {
        SettingsDialog(
            consoles = allConsoles,
            themeMode = themeMode,
            customPrimary = customPrimary,
            customOnPrimary = customOnPrimary,
            customBackground = customBackground,
            customSurface = customSurface,
            customSecondary = customSecondary,
            catalogViewMode = catalogViewMode,
            onSelectCatalogViewMode = { viewModel.setCatalogViewMode(it) },
            onToggleConsole = { slug, isEnabled ->
                viewModel.toggleConsoleEnabled(slug, isEnabled)
            },
            onSetAllConsoles = { isEnabled ->
                viewModel.setAllConsolesEnabled(isEnabled)
            },
            onMoveConsoleUp = { slug -> viewModel.moveConsoleUp(slug) },
            onMoveConsoleDown = { slug -> viewModel.moveConsoleDown(slug) },
            onMoveConsoleToTop = { slug -> viewModel.moveConsoleToTop(slug) },
            onResetConsolesOrder = { viewModel.resetConsolesOrder() },
            onSelectThemeMode = { viewModel.setThemeMode(it) },
            onApplyPreset = { viewModel.applyPreset(it) },
            onUpdateColor = { target, color -> viewModel.updateCustomColor(target, color) },
            onResetColors = { viewModel.resetColorsToDefault() },
            onDismiss = { viewModel.setShowManageConsoles(false) }
        )
    }

    // Multi-ROM Zip Extraction Dialog
    val currentPendingZip = pendingZipExtraction
    if (currentPendingZip != null) {
        ZipExtractionDialog(
            request = currentPendingZip,
            onExtractSelected = { selected ->
                viewModel.confirmZipExtraction(currentPendingZip, selected)
            },
            onKeepZip = {
                viewModel.keepZipWithoutExtraction(currentPendingZip)
            },
            onDismiss = {
                viewModel.dismissZipExtraction(currentPendingZip)
            }
        )
    }
}
