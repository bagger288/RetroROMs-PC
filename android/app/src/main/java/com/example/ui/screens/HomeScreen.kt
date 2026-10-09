package com.example.ui.screens

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.FilterList
import androidx.compose.material.icons.filled.Gamepad
import androidx.compose.material.icons.filled.GridView
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Tune
import androidx.compose.material.icons.filled.ViewList
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.R
import com.example.model.CatalogCategory
import com.example.model.ConsoleInfo
import com.example.model.GameCard
import com.example.ui.components.FullScreenImageDialog
import com.example.ui.components.GameCardItem
import com.example.ui.components.GameListItem
import com.example.ui.theme.ArcadeCardBorder
import com.example.ui.theme.ArcadeSurfaceContainer
import com.example.ui.theme.ArcadeSurfaceVariant
import com.example.ui.theme.CatalogViewMode
import com.example.ui.theme.GoldenAmber
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.OnPrimary
import com.example.ui.theme.RetroMagenta
import com.example.ui.theme.TextMuted
import com.example.ui.theme.TextPrimary
import com.example.ui.theme.TextSecondary
import com.example.ui.viewmodel.DownloadProgressState
import com.example.ui.viewmodel.GameSort
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    enabledConsoles: List<ConsoleInfo>,
    selectedConsole: ConsoleInfo?,
    selectedCategory: String,
    categories: List<CatalogCategory> = emptyList(),
    currentPage: Int,
    totalPages: Int = 1,
    hasNextPage: Boolean = false,
    hasPrevPage: Boolean = false,
    games: List<GameCard>,
    allSearchResults: List<GameCard>,
    isSearching: Boolean,
    searchPlatformFilter: String?,
    activeDownloads: Map<String, DownloadProgressState>,
    searchQuery: String,
    selectedSort: GameSort,
    isRefreshing: Boolean,
    onSelectConsole: (ConsoleInfo) -> Unit,
    onSelectCategory: (String) -> Unit,
    onGoToPage: (Int) -> Unit = {},
    onNextPage: () -> Unit = {},
    onPrevPage: () -> Unit = {},
    onAppendNextPage: () -> Unit = {},
    onLoadNextPage: () -> Unit = onNextPage,
    onSearchQueryChange: (String) -> Unit,
    onSelectSearchPlatformFilter: (String?) -> Unit,
    onSelectSort: (GameSort) -> Unit,
    onCardClick: (GameCard) -> Unit,
    onDownloadClick: (GameCard) -> Unit,
    onFavoriteClick: (GameCard) -> Unit,
    onOpenManageConsoles: () -> Unit,
    onRefresh: () -> Unit,
    catalogViewMode: CatalogViewMode = CatalogViewMode.GRID,
    onSelectCatalogViewMode: (CatalogViewMode) -> Unit = {},
    modifier: Modifier = Modifier
) {
    val coroutineScope = rememberCoroutineScope()
    val focusManager = LocalFocusManager.current
    var showSortMenu by remember { mutableStateOf(false) }
    var previewGame by remember { mutableStateOf<GameCard?>(null) }

    val gridState = rememberLazyGridState()
    var isFilterBarVisible by remember { mutableStateOf(true) }

    val nestedScrollConnection = remember {
        object : NestedScrollConnection {
            override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset {
                val delta = available.y
                if (delta < -8f && isFilterBarVisible && (gridState.firstVisibleItemIndex > 0 || gridState.firstVisibleItemScrollOffset > 25)) {
                    isFilterBarVisible = false
                } else if (delta > 8f && !isFilterBarVisible) {
                    isFilterBarVisible = true
                }
                return Offset.Zero
            }
        }
    }

    LaunchedEffect(gridState.firstVisibleItemIndex, gridState.firstVisibleItemScrollOffset) {
        if (gridState.firstVisibleItemIndex == 0 && gridState.firstVisibleItemScrollOffset < 15 && !isFilterBarVisible) {
            isFilterBarVisible = true
        }
    }

    LaunchedEffect(selectedCategory, selectedConsole) {
        isFilterBarVisible = true
        if (gridState.firstVisibleItemIndex > 0) {
            gridState.scrollToItem(0)
        }
    }

    val isSearchMode = searchQuery.isNotBlank()

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 16.dp)
    ) {
        // App Top Row
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 8.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Image(
                    painter = painterResource(id = R.drawable.ic_app_logo),
                    contentDescription = "RetroROMs App Icon",
                    modifier = Modifier
                        .size(38.dp)
                        .clip(RoundedCornerShape(9.dp))
                        .border(1.dp, ArcadeCardBorder, RoundedCornerShape(9.dp))
                )
                Spacer(modifier = Modifier.width(10.dp))
                Column {
                    Text(
                        text = "RetroROMs",
                        color = TextPrimary,
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Black,
                        letterSpacing = 0.5.sp
                    )
                    Text(
                        text = "Каталог Emu-Land.net",
                        color = TextMuted,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Medium
                    )
                }
            }

            IconButton(
                onClick = onRefresh,
                enabled = !isRefreshing,
                modifier = Modifier
                    .testTag("refresh_btn")
                    .size(50.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(ArcadeSurfaceVariant)
                    .border(1.dp, ArcadeCardBorder, RoundedCornerShape(12.dp))
            ) {
                if (isRefreshing) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(20.dp),
                        color = NeonCyan,
                        strokeWidth = 2.dp
                    )
                } else {
                    Icon(
                        imageVector = Icons.Default.Refresh,
                        contentDescription = "Обновить с Emu-Land",
                        tint = NeonCyan,
                        modifier = Modifier.size(20.dp)
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(4.dp))

        // Search Bar and Sort Row
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            OutlinedTextField(
                value = searchQuery,
                onValueChange = onSearchQueryChange,
                placeholder = {
                    Text(
                        text = "Поиск по всем платформам (Mario, Sonic…)",
                        color = TextMuted,
                        fontSize = 13.sp
                    )
                },
                leadingIcon = {
                    Icon(
                        imageVector = Icons.Default.Search,
                        contentDescription = "Search",
                        tint = if (isSearchMode) NeonCyan else TextMuted,
                        modifier = Modifier.size(18.dp)
                    )
                },
                trailingIcon = {
                    if (isSearchMode) {
                        IconButton(onClick = { onSearchQueryChange("") }) {
                            Icon(
                                imageVector = Icons.Default.Clear,
                                contentDescription = "Clear search",
                                tint = NeonCyan,
                                modifier = Modifier.size(18.dp)
                            )
                        }
                    }
                },
                singleLine = true,
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                keyboardActions = KeyboardActions(onSearch = { focusManager.clearFocus() }),
                shape = RoundedCornerShape(12.dp),
                colors = OutlinedTextFieldDefaults.colors(
                    focusedBorderColor = NeonCyan,
                    unfocusedBorderColor = if (isSearchMode) NeonCyan.copy(alpha = 0.5f) else ArcadeCardBorder,
                    focusedContainerColor = ArcadeSurfaceVariant,
                    unfocusedContainerColor = ArcadeSurfaceVariant,
                    focusedTextColor = TextPrimary,
                    unfocusedTextColor = TextPrimary
                ),
                modifier = Modifier
                    .weight(1f)
                    .height(50.dp)
                    .testTag("search_input")
            )

            Spacer(modifier = Modifier.width(8.dp))

            // Sort Dropdown
            Box {
                IconButton(
                    onClick = { showSortMenu = true },
                    modifier = Modifier
                        .testTag("sort_btn")
                        .size(50.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .background(ArcadeSurfaceVariant)
                        .border(1.dp, ArcadeCardBorder, RoundedCornerShape(12.dp))
                ) {
                    Icon(
                        imageVector = Icons.Default.FilterList,
                        contentDescription = "Sort games",
                        tint = GoldenAmber,
                        modifier = Modifier.size(20.dp)
                    )
                }

                DropdownMenu(
                    expanded = showSortMenu,
                    onDismissRequest = { showSortMenu = false },
                    modifier = Modifier.background(ArcadeSurfaceVariant)
                ) {
                    Text(
                        text = "СОРТИРОВКА",
                        color = GoldenAmber,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(horizontal = 14.dp, vertical = 6.dp)
                    )

                    DropdownMenuItem(
                        text = {
                            Text(
                                text = if (selectedSort == GameSort.RATING) "✓  По рейтингу" else "    По рейтингу",
                                color = if (selectedSort == GameSort.RATING) NeonCyan else TextPrimary,
                                fontWeight = if (selectedSort == GameSort.RATING) FontWeight.Bold else FontWeight.Normal
                            )
                        },
                        onClick = {
                            onSelectSort(GameSort.RATING)
                            showSortMenu = false
                        }
                    )
                    DropdownMenuItem(
                        text = {
                            Text(
                                text = if (selectedSort == GameSort.TITLE) "✓  По алфавиту (A-Z)" else "    По алфавиту (A-Z)",
                                color = if (selectedSort == GameSort.TITLE) NeonCyan else TextPrimary,
                                fontWeight = if (selectedSort == GameSort.TITLE) FontWeight.Bold else FontWeight.Normal
                            )
                        },
                        onClick = {
                            onSelectSort(GameSort.TITLE)
                            showSortMenu = false
                        }
                    )
                    DropdownMenuItem(
                        text = {
                            Text(
                                text = if (selectedSort == GameSort.YEAR) "✓  По году выпуска" else "    По году выпуска",
                                color = if (selectedSort == GameSort.YEAR) NeonCyan else TextPrimary,
                                fontWeight = if (selectedSort == GameSort.YEAR) FontWeight.Bold else FontWeight.Normal
                            )
                        },
                        onClick = {
                            onSelectSort(GameSort.YEAR)
                            showSortMenu = false
                        }
                    )

                    HorizontalDivider(color = ArcadeCardBorder, modifier = Modifier.padding(vertical = 4.dp))

                    Text(
                        text = "ВИД КАТАЛОГА",
                        color = GoldenAmber,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.padding(horizontal = 14.dp, vertical = 6.dp)
                    )

                    DropdownMenuItem(
                        text = {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Icon(
                                    imageVector = Icons.Default.GridView,
                                    contentDescription = null,
                                    tint = if (catalogViewMode == CatalogViewMode.GRID) NeonCyan else TextMuted,
                                    modifier = Modifier.size(18.dp)
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                                Text(
                                    text = if (catalogViewMode == CatalogViewMode.GRID) "✓  Сетка (по 2 игры)" else "    Сетка (по 2 игры)",
                                    color = if (catalogViewMode == CatalogViewMode.GRID) NeonCyan else TextPrimary,
                                    fontWeight = if (catalogViewMode == CatalogViewMode.GRID) FontWeight.Bold else FontWeight.Normal
                                )
                            }
                        },
                        onClick = {
                            onSelectCatalogViewMode(CatalogViewMode.GRID)
                            showSortMenu = false
                        }
                    )

                    DropdownMenuItem(
                        text = {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Icon(
                                    imageVector = Icons.Default.ViewList,
                                    contentDescription = null,
                                    tint = if (catalogViewMode == CatalogViewMode.LIST) NeonCyan else TextMuted,
                                    modifier = Modifier.size(18.dp)
                                )
                                Spacer(modifier = Modifier.width(8.dp))
                                Text(
                                    text = if (catalogViewMode == CatalogViewMode.LIST) "✓  Компактный список" else "    Компактный список",
                                    color = if (catalogViewMode == CatalogViewMode.LIST) NeonCyan else TextPrimary,
                                    fontWeight = if (catalogViewMode == CatalogViewMode.LIST) FontWeight.Bold else FontWeight.Normal
                                )
                            }
                        },
                        onClick = {
                            onSelectCatalogViewMode(CatalogViewMode.LIST)
                            showSortMenu = false
                        }
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(8.dp))

        if (isSearchMode) {
            // GLOBAL SEARCH MODE HEADER & PLATFORM FILTER CHIPS
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = if (isSearching) "Поиск на Emu-Land…" else "Найдено: ${allSearchResults.size} игр",
                        color = NeonCyan,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Bold
                    )
                    if (isSearching) {
                        Spacer(modifier = Modifier.width(8.dp))
                        CircularProgressIndicator(
                            modifier = Modifier.size(12.dp),
                            strokeWidth = 2.dp,
                            color = NeonCyan
                        )
                    }
                }

                Text(
                    text = "Сбросить",
                    color = TextMuted,
                    fontSize = 12.sp,
                    modifier = Modifier.clickable { onSearchQueryChange("") }
                )
            }

            AnimatedVisibility(
                visible = isFilterBarVisible,
                enter = expandVertically(animationSpec = tween(220)) + fadeIn(animationSpec = tween(180)),
                exit = shrinkVertically(animationSpec = tween(220)) + fadeOut(animationSpec = tween(180))
            ) {
                if (allSearchResults.isNotEmpty()) {
                    Column {
                        Spacer(modifier = Modifier.height(6.dp))
                        val platformsInResults = allSearchResults
                            .groupBy { it.consoleSlug }
                            .map { (slug, list) ->
                                val sample = list.first()
                                Triple(slug, sample.consoleName.split("/").first().trim(), list.size)
                            }

                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .horizontalScroll(rememberScrollState()),
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            val isAllSelected = searchPlatformFilter == null
                            Box(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(8.dp))
                                    .background(if (isAllSelected) NeonCyan else ArcadeSurfaceContainer)
                                    .border(1.dp, if (isAllSelected) NeonCyan else ArcadeCardBorder, RoundedCornerShape(8.dp))
                                    .clickable { onSelectSearchPlatformFilter(null) }
                                    .padding(horizontal = 9.dp, vertical = 5.dp)
                            ) {
                                Text(
                                    text = "Все платформы (${allSearchResults.size})",
                                    color = if (isAllSelected) OnPrimary else TextPrimary,
                                    fontSize = 11.sp,
                                    fontWeight = if (isAllSelected) FontWeight.Bold else FontWeight.Medium
                                )
                            }

                            platformsInResults.forEach { (slug, platformName, count) ->
                                val isSelected = searchPlatformFilter == slug
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(8.dp))
                                        .background(if (isSelected) NeonCyan else ArcadeSurfaceContainer)
                                        .border(1.dp, if (isSelected) NeonCyan else ArcadeCardBorder, RoundedCornerShape(8.dp))
                                        .clickable { onSelectSearchPlatformFilter(if (isSelected) null else slug) }
                                        .padding(horizontal = 9.dp, vertical = 5.dp)
                                ) {
                                    Text(
                                        text = "$platformName ($count)",
                                        color = if (isSelected) OnPrimary else TextPrimary,
                                        fontSize = 11.sp,
                                        fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium
                                    )
                                }
                            }
                        }
                    }
                }
            }

        } else {
            AnimatedVisibility(
                visible = isFilterBarVisible,
                enter = expandVertically(animationSpec = tween(220)) + fadeIn(animationSpec = tween(180)),
                exit = shrinkVertically(animationSpec = tween(220)) + fadeOut(animationSpec = tween(180))
            ) {
                Column(modifier = Modifier.fillMaxWidth()) {
                    // STANDARD BROWSE MODE: Platform Chips Row
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .horizontalScroll(rememberScrollState()),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        val currentSelected = selectedConsole ?: enabledConsoles.firstOrNull()

                        enabledConsoles.forEach { console ->
                            val isSelected = currentSelected?.slug == console.slug
                            Box(
                                modifier = Modifier
                                    .testTag("console_chip_${console.slug}")
                                    .clip(RoundedCornerShape(10.dp))
                                    .background(if (isSelected) NeonCyan else ArcadeSurfaceContainer)
                                    .border(
                                        1.dp,
                                        if (isSelected) NeonCyan else ArcadeCardBorder,
                                        RoundedCornerShape(10.dp)
                                    )
                                    .clickable { onSelectConsole(console) }
                                    .padding(horizontal = 12.dp, vertical = 7.dp)
                            ) {
                                Text(
                                    text = console.shortName,
                                    color = if (isSelected) OnPrimary else TextPrimary,
                                    fontSize = 12.sp,
                                    fontWeight = if (isSelected) FontWeight.Black else FontWeight.Medium
                                )
                            }
                        }

                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(10.dp))
                                .background(ArcadeSurfaceContainer)
                                .border(1.dp, ArcadeCardBorder, RoundedCornerShape(10.dp))
                                .clickable { onOpenManageConsoles() }
                                .padding(horizontal = 10.dp, vertical = 7.dp)
                        ) {
                            Text(
                                text = "+ Платформы",
                                color = NeonCyan,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(8.dp))

                    if (categories.size > 1) {
                        // Sub-Category / Alphabetical Filter Row ("Популярное", "Лучшие", etc.)
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .horizontalScroll(rememberScrollState()),
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            categories.forEach { cat ->
                                val isCatSelected = selectedCategory.equals(cat.key, ignoreCase = true)
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(8.dp))
                                        .background(if (isCatSelected) GoldenAmber else ArcadeSurfaceContainer)
                                        .border(1.dp, if (isCatSelected) GoldenAmber else ArcadeCardBorder, RoundedCornerShape(8.dp))
                                        .clickable { onSelectCategory(cat.key) }
                                        .padding(horizontal = 9.dp, vertical = 5.dp)
                                ) {
                                    Text(
                                        text = cat.label,
                                        color = if (isCatSelected) OnPrimary else TextPrimary,
                                        fontSize = 11.sp,
                                        fontWeight = if (isCatSelected) FontWeight.Bold else FontWeight.Medium
                                    )
                                }
                            }
                        }

                        Spacer(modifier = Modifier.height(8.dp))
                    }
                }
            }

            // Results info row
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                val activeConsoleName = (selectedConsole ?: enabledConsoles.firstOrNull())?.shortName ?: "Игры"
                val categoryName = categories.firstOrNull { it.key.equals(selectedCategory, ignoreCase = true) }?.label
                    ?: if (selectedCategory.equals("all", ignoreCase = true)) "Все игры" else selectedCategory
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = if (isSearchMode) "Результаты поиска (${games.size})" else "$activeConsoleName • $categoryName (${games.size})",
                        color = TextSecondary,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                    if (isRefreshing) {
                        Spacer(modifier = Modifier.width(8.dp))
                        CircularProgressIndicator(
                            modifier = Modifier.size(12.dp),
                            strokeWidth = 1.8.dp,
                            color = NeonCyan
                        )
                    }
                }

                if (!isSearchMode) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        Text(
                            text = if (totalPages > 1) "Стр. $currentPage из $totalPages" else "Стр. $currentPage",
                            color = TextMuted,
                            fontSize = 11.sp
                        )
                        if (totalPages > 1) {
                            IconButton(
                                onClick = {
                                    onPrevPage()
                                    coroutineScope.launch { gridState.animateScrollToItem(0) }
                                },
                                enabled = (hasPrevPage || currentPage > 1) && !isRefreshing,
                                modifier = Modifier.size(24.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                                    contentDescription = "Предыдущая страница",
                                    tint = if (hasPrevPage || currentPage > 1) NeonCyan else TextMuted.copy(alpha = 0.35f),
                                    modifier = Modifier.size(14.dp)
                                )
                            }
                            IconButton(
                                onClick = {
                                    onNextPage()
                                    coroutineScope.launch { gridState.animateScrollToItem(0) }
                                },
                                enabled = (hasNextPage || currentPage < totalPages) && !isRefreshing,
                                modifier = Modifier.size(24.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                                    contentDescription = "Следующая страница",
                                    tint = if (hasNextPage || currentPage < totalPages) NeonCyan else TextMuted.copy(alpha = 0.35f),
                                    modifier = Modifier.size(14.dp)
                                )
                            }
                        }
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(8.dp))

        // Games Grid
        if (games.isEmpty()) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f),
                contentAlignment = Alignment.Center
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    if (isSearching || isRefreshing) {
                        CircularProgressIndicator(
                            color = NeonCyan,
                            modifier = Modifier.size(36.dp)
                        )
                        Spacer(modifier = Modifier.height(12.dp))
                        Text(
                            text = if (isSearching) "Глобальный поиск по всем консолям Emu-Land…" else "Загрузка игр с Emu-Land.net…",
                            color = TextPrimary,
                            fontSize = 15.sp,
                            fontWeight = FontWeight.Medium
                        )
                    } else {
                        Icon(
                            imageVector = Icons.Default.Search,
                            contentDescription = null,
                            tint = TextMuted,
                            modifier = Modifier.size(48.dp)
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = if (isSearchMode) "Ничего не найдено по запросу «$searchQuery»" else "Игры для выбранной категории подгружаются",
                            color = TextPrimary,
                            fontSize = 15.sp,
                            fontWeight = FontWeight.Medium
                        )
                        if (isSearchMode) {
                            Spacer(modifier = Modifier.height(4.dp))
                            Text(
                                text = "Попробуйте ввести название на английском (например: Mario, Sonic, Mortal Kombat, Zelda, Contra).",
                                color = TextMuted,
                                fontSize = 12.sp,
                                textAlign = TextAlign.Center,
                                modifier = Modifier.padding(horizontal = 24.dp)
                            )
                            Spacer(modifier = Modifier.height(12.dp))
                            Button(
                                onClick = { onSearchQueryChange("") },
                                colors = ButtonDefaults.buttonColors(
                                    containerColor = ArcadeSurfaceVariant,
                                    contentColor = NeonCyan
                                )
                            ) {
                                Text("Очистить поиск")
                            }
                        }
                    }
                }
            }
        } else {
            val isGrid = catalogViewMode == CatalogViewMode.GRID
            val columnCount = if (isGrid) 2 else 1

            LazyVerticalGrid(
                state = gridState,
                columns = GridCells.Fixed(columnCount),
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f)
                    .nestedScroll(nestedScrollConnection),
                contentPadding = PaddingValues(bottom = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalArrangement = Arrangement.spacedBy(if (isGrid) 10.dp else 4.dp)
            ) {
                items(games, key = { it.id }) { game ->
                    val progress = activeDownloads[game.id]
                    if (isGrid) {
                        GameCardItem(
                            game = game,
                            downloadProgress = progress,
                            onCardClick = { onCardClick(game) },
                            onLongClick = { previewGame = game },
                            onDownloadClick = { onDownloadClick(game) },
                            onFavoriteClick = { onFavoriteClick(game) }
                        )
                    } else {
                        GameListItem(
                            game = game,
                            downloadProgress = progress,
                            onCardClick = { onCardClick(game) },
                            onLongClick = { previewGame = game },
                            onDownloadClick = { onDownloadClick(game) },
                            onFavoriteClick = { onFavoriteClick(game) }
                        )
                    }
                }

                // Pagination controls at bottom of grid
                if (!isSearchMode && (totalPages > 1 || hasNextPage || currentPage > 1 || games.size >= 15)) {
                    item(span = { GridItemSpan(columnCount) }) {
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 16.dp),
                            horizontalAlignment = Alignment.CenterHorizontally
                        ) {
                            Text(
                                text = if (totalPages > 1) "СТРАНИЦА $currentPage ИЗ $totalPages" else "СТРАНИЦА $currentPage",
                                color = GoldenAmber,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                                letterSpacing = 1.sp
                            )
                            Spacer(modifier = Modifier.height(10.dp))

                            // Main Navigation Controls: Prev [1] [2] [3]... Next
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .horizontalScroll(rememberScrollState()),
                                horizontalArrangement = Arrangement.Center,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                // Previous button
                                val canGoPrev = (hasPrevPage || currentPage > 1) && !isRefreshing
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(8.dp))
                                        .background(if (canGoPrev) ArcadeSurfaceVariant else ArcadeSurfaceContainer.copy(alpha = 0.5f))
                                        .border(1.dp, if (canGoPrev) ArcadeCardBorder else Color.Transparent, RoundedCornerShape(8.dp))
                                        .clickable(enabled = canGoPrev) {
                                            onPrevPage()
                                            coroutineScope.launch { gridState.animateScrollToItem(0) }
                                        }
                                        .padding(horizontal = 10.dp, vertical = 8.dp)
                                ) {
                                    Row(verticalAlignment = Alignment.CenterVertically) {
                                        Icon(
                                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                                            contentDescription = null,
                                            tint = if (canGoPrev) NeonCyan else TextMuted.copy(alpha = 0.4f),
                                            modifier = Modifier.size(14.dp)
                                        )
                                        Spacer(modifier = Modifier.width(4.dp))
                                        Text(
                                            text = "Назад",
                                            color = if (canGoPrev) TextPrimary else TextMuted.copy(alpha = 0.4f),
                                            fontSize = 12.sp,
                                            fontWeight = FontWeight.SemiBold
                                        )
                                    }
                                }

                                Spacer(modifier = Modifier.width(6.dp))

                                // Numbered pills: 1, 2, 3...
                                val pageNumbers = remember(currentPage, totalPages) {
                                    val pages = mutableListOf<Int?>()
                                    if (totalPages <= 6) {
                                        for (p in 1..totalPages) pages.add(p)
                                    } else {
                                        pages.add(1)
                                        val start = (currentPage - 1).coerceAtLeast(2)
                                        val end = (currentPage + 1).coerceAtMost(totalPages - 1)
                                        if (start > 2) pages.add(null) // separator
                                        for (p in start..end) pages.add(p)
                                        if (end < totalPages - 1) pages.add(null) // separator
                                        pages.add(totalPages)
                                    }
                                    pages
                                }

                                pageNumbers.forEach { pNum ->
                                    if (pNum == null) {
                                        Text(
                                            text = "…",
                                            color = TextMuted,
                                            fontSize = 13.sp,
                                            fontWeight = FontWeight.Bold,
                                            modifier = Modifier.padding(horizontal = 4.dp)
                                        )
                                    } else {
                                        val isCurrent = pNum == currentPage
                                        Box(
                                            modifier = Modifier
                                                .padding(horizontal = 2.dp)
                                                .size(34.dp)
                                                .clip(RoundedCornerShape(8.dp))
                                                .background(if (isCurrent) GoldenAmber else ArcadeSurfaceContainer)
                                                .border(
                                                    width = 1.dp,
                                                    color = if (isCurrent) GoldenAmber else ArcadeCardBorder,
                                                    shape = RoundedCornerShape(8.dp)
                                                )
                                                .clickable(enabled = !isCurrent && !isRefreshing) {
                                                    onGoToPage(pNum)
                                                    coroutineScope.launch { gridState.animateScrollToItem(0) }
                                                },
                                            contentAlignment = Alignment.Center
                                        ) {
                                            Text(
                                                text = pNum.toString(),
                                                color = if (isCurrent) OnPrimary else TextPrimary,
                                                fontSize = 12.sp,
                                                fontWeight = if (isCurrent) FontWeight.Bold else FontWeight.Medium
                                            )
                                        }
                                    }
                                }

                                Spacer(modifier = Modifier.width(6.dp))

                                // Next button
                                val canGoNext = (hasNextPage || currentPage < totalPages) && !isRefreshing
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(8.dp))
                                        .background(if (canGoNext) ArcadeSurfaceVariant else ArcadeSurfaceContainer.copy(alpha = 0.5f))
                                        .border(1.dp, if (canGoNext) ArcadeCardBorder else Color.Transparent, RoundedCornerShape(8.dp))
                                        .clickable(enabled = canGoNext) {
                                            onNextPage()
                                            coroutineScope.launch { gridState.animateScrollToItem(0) }
                                        }
                                        .padding(horizontal = 10.dp, vertical = 8.dp)
                                ) {
                                    Row(verticalAlignment = Alignment.CenterVertically) {
                                        Text(
                                            text = "Вперёд",
                                            color = if (canGoNext) TextPrimary else TextMuted.copy(alpha = 0.4f),
                                            fontSize = 12.sp,
                                            fontWeight = FontWeight.SemiBold
                                        )
                                        Spacer(modifier = Modifier.width(4.dp))
                                        Icon(
                                            imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                                            contentDescription = null,
                                            tint = if (canGoNext) NeonCyan else TextMuted.copy(alpha = 0.4f),
                                            modifier = Modifier.size(14.dp)
                                        )
                                    }
                                }
                            }

                            // Optional "Загрузить ещё к списку" button (if hasNextPage)
                            if (hasNextPage || currentPage < totalPages) {
                                Spacer(modifier = Modifier.height(12.dp))
                                Button(
                                    onClick = onAppendNextPage,
                                    enabled = !isRefreshing,
                                    modifier = Modifier
                                        .fillMaxWidth(0.85f)
                                        .height(42.dp)
                                        .testTag("load_more_btn"),
                                    shape = RoundedCornerShape(10.dp),
                                    colors = ButtonDefaults.buttonColors(
                                        containerColor = ArcadeSurfaceContainer,
                                        contentColor = NeonCyan
                                    ),
                                    border = androidx.compose.foundation.BorderStroke(1.dp, ArcadeCardBorder)
                                ) {
                                    if (isRefreshing) {
                                        CircularProgressIndicator(
                                            modifier = Modifier.size(16.dp),
                                            color = NeonCyan,
                                            strokeWidth = 2.dp
                                        )
                                        Spacer(modifier = Modifier.width(8.dp))
                                        Text("Загрузка с Emu-Land…", fontSize = 12.sp)
                                    } else {
                                        Text(
                                            text = "+ Добавить страницу ${currentPage + 1} к списку",
                                            fontWeight = FontWeight.Medium,
                                            fontSize = 12.sp
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fullscreen Screenshot / Cover Preview Dialog
    if (previewGame != null) {
        FullScreenImageDialog(
            imageUrl = previewGame?.coverUrl,
            title = previewGame?.title ?: "",
            subtitle = previewGame?.consoleName,
            onDismiss = { previewGame = null }
        )
    }
}
