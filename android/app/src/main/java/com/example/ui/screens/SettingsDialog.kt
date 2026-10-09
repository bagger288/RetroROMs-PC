package com.example.ui.screens

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.ColorLens
import androidx.compose.material.icons.filled.DarkMode
import androidx.compose.material.icons.filled.Gamepad
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.LightMode
import androidx.compose.material.icons.filled.Palette
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.VerticalAlignTop
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Tab
import androidx.compose.material3.TabRow
import androidx.compose.material3.TabRowDefaults
import androidx.compose.material3.TabRowDefaults.tabIndicatorOffset
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.example.model.ConsoleInfo
import com.example.ui.theme.ArcadeCardBorder
import com.example.ui.theme.ArcadeDarkBg
import com.example.ui.theme.ArcadeSurface
import com.example.ui.theme.ArcadeSurfaceContainer
import com.example.ui.theme.ArcadeSurfaceVariant
import com.example.ui.theme.CatalogViewMode
import com.example.ui.theme.DefaultDarkTheme
import com.example.ui.theme.DefaultLightTheme
import com.example.ui.theme.GoldenAmber
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.OnPrimary
import com.example.ui.theme.PredefinedPresets
import com.example.ui.theme.RetroMagenta
import com.example.ui.theme.TextMuted
import com.example.ui.theme.TextPrimary
import com.example.ui.theme.TextSecondary
import com.example.ui.theme.ThemeMode
import com.example.ui.theme.ThemePreset

enum class ColorTarget {
    PRIMARY,
    ON_PRIMARY,
    SECONDARY,
    BACKGROUND,
    SURFACE
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsDialog(
    consoles: List<ConsoleInfo>,
    themeMode: ThemeMode,
    customPrimary: Color,
    customOnPrimary: Color,
    customBackground: Color,
    customSurface: Color,
    customSecondary: Color,
    catalogViewMode: CatalogViewMode = CatalogViewMode.GRID,
    onToggleConsole: (String, Boolean) -> Unit,
    onSetAllConsoles: (Boolean) -> Unit,
    onMoveConsoleUp: (String) -> Unit = {},
    onMoveConsoleDown: (String) -> Unit = {},
    onMoveConsoleToTop: (String) -> Unit = {},
    onResetConsolesOrder: () -> Unit = {},
    onSelectThemeMode: (ThemeMode) -> Unit,
    onSelectCatalogViewMode: (CatalogViewMode) -> Unit = {},
    onApplyPreset: (ThemePreset) -> Unit,
    onUpdateColor: (ColorTarget, Color) -> Unit,
    onResetColors: () -> Unit,
    onDismiss: () -> Unit
) {
    var selectedTab by remember { mutableIntStateOf(0) } // 0: Платформы, 1: Тема и цвета
    var activeColorTarget by remember { mutableStateOf(ColorTarget.PRIMARY) }

    val quickColors = listOf(
        Color(0xFF00E5FF), Color(0xFF0284C7), Color(0xFF0089CF), Color(0xFF6366F1),
        Color(0xFF8B5CF6), Color(0xFFD946EF), Color(0xFFFF3366), Color(0xFFE60012),
        Color(0xFFF97316), Color(0xFFFFB300), Color(0xFF8BAC0F), Color(0xFF00E676),
        Color(0xFF10B981), Color(0xFF14B8A6), Color(0xFF0B0E14), Color(0xFF141923),
        Color(0xFF1E2535), Color(0xFF000000), Color(0xFF1E1E2E), Color(0xFFF6F8FC),
        Color(0xFFFFFFFF), Color(0xFFE2E8F0), Color(0xFF334155), Color(0xFF475569)
    )

    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false)
    ) {
        Surface(
            modifier = Modifier
                .fillMaxWidth(0.94f)
                .fillMaxHeight(0.88f)
                .clip(RoundedCornerShape(20.dp))
                .border(1.dp, ArcadeCardBorder, RoundedCornerShape(20.dp)),
            color = ArcadeSurface
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(20.dp)
            ) {
                // Header
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(
                            modifier = Modifier
                                .size(36.dp)
                                .clip(RoundedCornerShape(10.dp))
                                .background(NeonCyan.copy(alpha = 0.15f)),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                imageVector = if (selectedTab == 0) Icons.Default.Gamepad else Icons.Default.Palette,
                                contentDescription = null,
                                tint = NeonCyan,
                                modifier = Modifier.size(20.dp)
                            )
                        }
                        Spacer(modifier = Modifier.width(10.dp))
                        Column {
                            Text(
                                text = "Настройки",
                                color = TextPrimary,
                                fontSize = 17.sp,
                                fontWeight = FontWeight.Bold
                            )
                            Text(
                                text = "Платформы и внешний вид",
                                color = TextMuted,
                                fontSize = 11.sp
                            )
                        }
                    }

                    IconButton(
                        onClick = onDismiss,
                        modifier = Modifier
                            .size(34.dp)
                            .clip(RoundedCornerShape(8.dp))
                            .background(ArcadeSurfaceVariant)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Close,
                            contentDescription = "Close",
                            tint = TextSecondary,
                            modifier = Modifier.size(18.dp)
                        )
                    }
                }

                Spacer(modifier = Modifier.height(14.dp))

                // Tabs: Платформы vs Тема и Цвета
                TabRow(
                    selectedTabIndex = selectedTab,
                    containerColor = ArcadeSurfaceVariant,
                    contentColor = NeonCyan,
                    indicator = { tabPositions ->
                        TabRowDefaults.SecondaryIndicator(
                            modifier = Modifier.tabIndicatorOffset(tabPositions[selectedTab]),
                            color = NeonCyan,
                            height = 3.dp
                        )
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(12.dp))
                ) {
                    Tab(
                        selected = selectedTab == 0,
                        onClick = { selectedTab = 0 },
                        text = {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Icon(Icons.Default.Gamepad, contentDescription = null, modifier = Modifier.size(16.dp))
                                Spacer(modifier = Modifier.width(6.dp))
                                Text("Платформы (${consoles.count { it.isEnabled }})", fontWeight = FontWeight.Bold, fontSize = 13.sp)
                            }
                        }
                    )
                    Tab(
                        selected = selectedTab == 1,
                        onClick = { selectedTab = 1 },
                        text = {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Icon(Icons.Default.Palette, contentDescription = null, modifier = Modifier.size(16.dp))
                                Spacer(modifier = Modifier.width(6.dp))
                                Text("Тема и Цвета", fontWeight = FontWeight.Bold, fontSize = 13.sp)
                            }
                        }
                    )
                }

                Spacer(modifier = Modifier.height(14.dp))

                // Tab Content
                if (selectedTab == 0) {
                    // TAB 0: CONSOLES MANAGEMENT
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = "Платформы и их порядок:",
                            color = TextSecondary,
                            fontSize = 12.sp,
                            fontWeight = FontWeight.Medium
                        )

                        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            Text(
                                text = "Все",
                                color = NeonCyan,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier
                                    .clip(RoundedCornerShape(6.dp))
                                    .background(ArcadeSurfaceVariant)
                                    .clickable { onSetAllConsoles(true) }
                                    .padding(horizontal = 7.dp, vertical = 4.dp)
                            )
                            Text(
                                text = "Снять",
                                color = TextMuted,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Medium,
                                modifier = Modifier
                                    .clip(RoundedCornerShape(6.dp))
                                    .background(ArcadeSurfaceVariant)
                                    .clickable { onSetAllConsoles(false) }
                                    .padding(horizontal = 7.dp, vertical = 4.dp)
                            )
                            Text(
                                text = "Сброс",
                                color = GoldenAmber,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier
                                    .clip(RoundedCornerShape(6.dp))
                                    .background(ArcadeSurfaceVariant)
                                    .clickable { onResetConsolesOrder() }
                                    .padding(horizontal = 7.dp, vertical = 4.dp)
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(4.dp))

                    Text(
                        text = "Используйте ▲ ▼ или ⤒ (в топ), чтобы поднять избранные консоли. Они будут первыми на главном экране!",
                        color = TextMuted,
                        fontSize = 10.sp,
                        lineHeight = 13.sp
                    )

                    Spacer(modifier = Modifier.height(8.dp))

                    LazyColumn(
                        modifier = Modifier
                            .fillMaxWidth()
                            .weight(1f),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        itemsIndexed(consoles, key = { _, console -> console.slug }) { index, console ->
                            var isMarquee by remember(console.slug) { mutableStateOf(false) }

                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clip(RoundedCornerShape(12.dp))
                                    .background(if (console.isEnabled) ArcadeSurfaceVariant else ArcadeSurfaceContainer.copy(alpha = 0.5f))
                                    .border(
                                        1.dp,
                                        if (console.isEnabled) NeonCyan.copy(alpha = 0.35f) else ArcadeCardBorder,
                                        RoundedCornerShape(12.dp)
                                    )
                                    .clickable { onToggleConsole(console.slug, !console.isEnabled) }
                                    .padding(horizontal = 8.dp, vertical = 8.dp),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    modifier = Modifier.weight(1f)
                                ) {
                                    // Order Badge (#1, #2...)
                                    Box(
                                        modifier = Modifier
                                            .size(width = 24.dp, height = 24.dp)
                                            .clip(RoundedCornerShape(6.dp))
                                            .background(ArcadeSurfaceContainer),
                                        contentAlignment = Alignment.Center
                                    ) {
                                        Text(
                                            text = "${index + 1}",
                                            fontSize = 11.sp,
                                            fontWeight = FontWeight.Bold,
                                            color = if (console.isEnabled) TextSecondary else TextMuted
                                        )
                                    }

                                    Spacer(modifier = Modifier.width(10.dp))

                                    Column(modifier = Modifier.weight(1f)) {
                                        Text(
                                            text = console.name,
                                            color = if (console.isEnabled) TextPrimary else TextMuted,
                                            fontSize = 13.sp,
                                            fontWeight = FontWeight.SemiBold,
                                            maxLines = 1,
                                            overflow = if (isMarquee) TextOverflow.Clip else TextOverflow.Ellipsis,
                                            modifier = Modifier
                                                .fillMaxWidth()
                                                .then(
                                                    if (isMarquee) {
                                                        Modifier.basicMarquee(
                                                            iterations = 4,
                                                            initialDelayMillis = 200,
                                                            repeatDelayMillis = 800,
                                                            velocity = 32.dp
                                                        )
                                                    } else Modifier
                                                )
                                                .clickable(
                                                    interactionSource = remember { MutableInteractionSource() },
                                                    indication = null
                                                ) {
                                                    isMarquee = !isMarquee
                                                }
                                        )
                                        Spacer(modifier = Modifier.height(1.dp))
                                        Text(
                                            text = "${console.category} • ${console.releaseYear}",
                                            color = TextMuted,
                                            fontSize = 10.sp,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                    }
                                }

                                Spacer(modifier = Modifier.width(4.dp))

                                // Reorder actions & Switch
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(1.dp)
                                ) {
                                    if (index > 0) {
                                        IconButton(
                                            onClick = { onMoveConsoleToTop(console.slug) },
                                            modifier = Modifier.size(28.dp)
                                        ) {
                                            Icon(
                                                imageVector = Icons.Default.VerticalAlignTop,
                                                contentDescription = "В самый верх",
                                                tint = GoldenAmber,
                                                modifier = Modifier.size(16.dp)
                                            )
                                        }
                                    } else {
                                        Spacer(modifier = Modifier.size(28.dp))
                                    }

                                    IconButton(
                                        onClick = { onMoveConsoleUp(console.slug) },
                                        enabled = index > 0,
                                        modifier = Modifier.size(28.dp)
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.KeyboardArrowUp,
                                            contentDescription = "Вверх",
                                            tint = if (index > 0) NeonCyan else TextMuted.copy(alpha = 0.2f),
                                            modifier = Modifier.size(18.dp)
                                        )
                                    }

                                    IconButton(
                                        onClick = { onMoveConsoleDown(console.slug) },
                                        enabled = index < consoles.size - 1,
                                        modifier = Modifier.size(28.dp)
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.KeyboardArrowDown,
                                            contentDescription = "Вниз",
                                            tint = if (index < consoles.size - 1) NeonCyan else TextMuted.copy(alpha = 0.2f),
                                            modifier = Modifier.size(18.dp)
                                        )
                                    }

                                    Switch(
                                        checked = console.isEnabled,
                                        onCheckedChange = { onToggleConsole(console.slug, it) },
                                        colors = SwitchDefaults.colors(
                                            checkedThumbColor = OnPrimary,
                                            checkedTrackColor = NeonCyan,
                                            uncheckedThumbColor = TextMuted,
                                            uncheckedTrackColor = ArcadeSurfaceContainer
                                        ),
                                        modifier = Modifier.scale(0.75f)
                                    )
                                }
                            }
                        }
                    }

                } else {
                    // TAB 1: THEME & COLOR PICKER
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .weight(1f)
                            .verticalScroll(rememberScrollState())
                    ) {
                        // Theme Mode Switcher (Dark / Light / Custom)
                        Text(
                            text = "РЕЖИМ ОФОРМЛЕНИЯ",
                            color = GoldenAmber,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold,
                            letterSpacing = 0.5.sp
                        )
                        Spacer(modifier = Modifier.height(8.dp))

                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            ThemeModeCard(
                                title = "Тёмная",
                                icon = Icons.Default.DarkMode,
                                isSelected = themeMode == ThemeMode.DARK,
                                onClick = { onSelectThemeMode(ThemeMode.DARK) },
                                modifier = Modifier.weight(1f)
                            )
                            ThemeModeCard(
                                title = "Светлая",
                                icon = Icons.Default.LightMode,
                                isSelected = themeMode == ThemeMode.LIGHT,
                                onClick = { onSelectThemeMode(ThemeMode.LIGHT) },
                                modifier = Modifier.weight(1f)
                            )
                            ThemeModeCard(
                                title = "Своя (RGB)",
                                icon = Icons.Default.ColorLens,
                                isSelected = themeMode == ThemeMode.CUSTOM,
                                onClick = { onSelectThemeMode(ThemeMode.CUSTOM) },
                                modifier = Modifier.weight(1f)
                            )
                        }

                        Spacer(modifier = Modifier.height(16.dp))

                        // Theme Presets
                        Text(
                            text = "ГОТОВЫЕ ЦВЕТОВЫЕ ПРЕСЕТЫ",
                            color = GoldenAmber,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold,
                            letterSpacing = 0.5.sp
                        )
                        Spacer(modifier = Modifier.height(8.dp))

                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .horizontalScroll(rememberScrollState()),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            PredefinedPresets.forEach { preset ->
                                Column(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(12.dp))
                                        .background(ArcadeSurfaceVariant)
                                        .border(1.dp, ArcadeCardBorder, RoundedCornerShape(12.dp))
                                        .clickable { onApplyPreset(preset) }
                                        .padding(horizontal = 12.dp, vertical = 10.dp),
                                    horizontalAlignment = Alignment.CenterHorizontally
                                ) {
                                    Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                        Box(
                                            modifier = Modifier
                                                .size(16.dp)
                                                .clip(CircleShape)
                                                .background(preset.primary)
                                        )
                                        Box(
                                            modifier = Modifier
                                                .size(16.dp)
                                                .clip(CircleShape)
                                                .background(preset.secondary)
                                        )
                                        Box(
                                            modifier = Modifier
                                                .size(16.dp)
                                                .clip(CircleShape)
                                                .background(preset.background)
                                                .border(1.dp, Color.White.copy(alpha = 0.2f), CircleShape)
                                        )
                                    }
                                    Spacer(modifier = Modifier.height(6.dp))
                                    Text(
                                        text = preset.name,
                                        color = TextPrimary,
                                        fontSize = 11.sp,
                                        fontWeight = FontWeight.SemiBold
                                    )
                                }
                            }
                        }

                        Spacer(modifier = Modifier.height(16.dp))

                        // Catalog View Mode & List Styles
                        Text(
                            text = "ВИД КАТАЛОГА И КАРТОЧЕК",
                            color = GoldenAmber,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold,
                            letterSpacing = 0.5.sp
                        )
                        Spacer(modifier = Modifier.height(8.dp))

                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Box(
                                modifier = Modifier
                                    .weight(1f)
                                    .clip(RoundedCornerShape(10.dp))
                                    .background(if (catalogViewMode == CatalogViewMode.GRID) NeonCyan else ArcadeSurfaceVariant)
                                    .border(1.dp, if (catalogViewMode == CatalogViewMode.GRID) NeonCyan else ArcadeCardBorder, RoundedCornerShape(10.dp))
                                    .clickable { onSelectCatalogViewMode(CatalogViewMode.GRID) }
                                    .padding(vertical = 10.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                Text(
                                    text = "Сетка (по 2)",
                                    color = if (catalogViewMode == CatalogViewMode.GRID) OnPrimary else TextPrimary,
                                    fontSize = 12.sp,
                                    fontWeight = FontWeight.Bold
                                )
                            }

                            Box(
                                modifier = Modifier
                                    .weight(1f)
                                    .clip(RoundedCornerShape(10.dp))
                                    .background(if (catalogViewMode == CatalogViewMode.LIST) NeonCyan else ArcadeSurfaceVariant)
                                    .border(1.dp, if (catalogViewMode == CatalogViewMode.LIST) NeonCyan else ArcadeCardBorder, RoundedCornerShape(10.dp))
                                    .clickable { onSelectCatalogViewMode(CatalogViewMode.LIST) }
                                    .padding(vertical = 10.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                Text(
                                    text = "Список (1 колонка)",
                                    color = if (catalogViewMode == CatalogViewMode.LIST) OnPrimary else TextPrimary,
                                    fontSize = 12.sp,
                                    fontWeight = FontWeight.Bold
                                )
                            }
                        }

                        Spacer(modifier = Modifier.height(16.dp))

                        // Interactive Color Picker for Custom Mode
                        Text(
                            text = "ТОЧНАЯ НАСТРОЙКА ЦВЕТОВ",
                            color = GoldenAmber,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold,
                            letterSpacing = 0.5.sp
                        )
                        Spacer(modifier = Modifier.height(8.dp))

                        // Target Selector: Primary, OnPrimary, Secondary, Background, Surface
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .horizontalScroll(rememberScrollState()),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            TargetPill(
                                label = "Акцент",
                                color = customPrimary,
                                isSelected = activeColorTarget == ColorTarget.PRIMARY,
                                onClick = { activeColorTarget = ColorTarget.PRIMARY }
                            )
                            TargetPill(
                                label = "Кнопки/Иконки",
                                color = customOnPrimary,
                                isSelected = activeColorTarget == ColorTarget.ON_PRIMARY,
                                onClick = { activeColorTarget = ColorTarget.ON_PRIMARY }
                            )
                            TargetPill(
                                label = "Вторичный",
                                color = customSecondary,
                                isSelected = activeColorTarget == ColorTarget.SECONDARY,
                                onClick = { activeColorTarget = ColorTarget.SECONDARY }
                            )
                            TargetPill(
                                label = "Фон",
                                color = customBackground,
                                isSelected = activeColorTarget == ColorTarget.BACKGROUND,
                                onClick = { activeColorTarget = ColorTarget.BACKGROUND }
                            )
                            TargetPill(
                                label = "Карточки",
                                color = customSurface,
                                isSelected = activeColorTarget == ColorTarget.SURFACE,
                                onClick = { activeColorTarget = ColorTarget.SURFACE }
                            )
                        }

                        Spacer(modifier = Modifier.height(12.dp))

                        val activeColor = when (activeColorTarget) {
                            ColorTarget.PRIMARY -> customPrimary
                            ColorTarget.ON_PRIMARY -> customOnPrimary
                            ColorTarget.SECONDARY -> customSecondary
                            ColorTarget.BACKGROUND -> customBackground
                            ColorTarget.SURFACE -> customSurface
                        }

                        // Current Color Preview and Hex
                        val hexString = String.format("#%06X", 0xFFFFFF and activeColor.toArgb())
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(12.dp))
                                .background(ArcadeSurfaceContainer)
                                .padding(12.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.SpaceBetween
                        ) {
                            Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.weight(1f)) {
                                Box(
                                    modifier = Modifier
                                        .size(36.dp)
                                        .clip(RoundedCornerShape(8.dp))
                                        .background(activeColor)
                                        .border(1.dp, Color.White.copy(alpha = 0.3f), RoundedCornerShape(8.dp))
                                )
                                Spacer(modifier = Modifier.width(12.dp))
                                Column {
                                    Text(
                                        text = when (activeColorTarget) {
                                            ColorTarget.PRIMARY -> "Основной акцент"
                                            ColorTarget.ON_PRIMARY -> "Иконки и текст на кнопках"
                                            ColorTarget.SECONDARY -> "Вторичный цвет"
                                            ColorTarget.BACKGROUND -> "Фон приложения"
                                            ColorTarget.SURFACE -> "Цвет карточек"
                                        },
                                        color = TextPrimary,
                                        fontSize = 13.sp,
                                        fontWeight = FontWeight.Bold
                                    )
                                    Text(
                                        text = when (activeColorTarget) {
                                            ColorTarget.ON_PRIMARY -> "$hexString • стрелки, кнопки, переключатели"
                                            else -> hexString
                                        },
                                        color = NeonCyan,
                                        fontSize = 12.sp,
                                        fontWeight = FontWeight.SemiBold
                                    )
                                }
                            }

                            Button(
                                onClick = onResetColors,
                                colors = ButtonDefaults.buttonColors(
                                    containerColor = ArcadeSurfaceVariant,
                                    contentColor = TextSecondary
                                ),
                                contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 10.dp, vertical = 6.dp),
                                shape = RoundedCornerShape(8.dp)
                            ) {
                                Icon(Icons.Default.Refresh, contentDescription = null, modifier = Modifier.size(14.dp))
                                Spacer(modifier = Modifier.width(4.dp))
                                Text("Сброс", fontSize = 11.sp)
                            }
                        }

                        Spacer(modifier = Modifier.height(12.dp))

                        // Quick Palette Grid (24 colors)
                        Text(
                            text = "Быстрая палитра:",
                            color = TextSecondary,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.SemiBold
                        )
                        Spacer(modifier = Modifier.height(6.dp))

                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .horizontalScroll(rememberScrollState()),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            quickColors.forEach { col ->
                                Box(
                                    modifier = Modifier
                                        .size(30.dp)
                                        .clip(CircleShape)
                                        .background(col)
                                        .border(
                                            if (col.toArgb() == activeColor.toArgb()) 2.dp else 1.dp,
                                            if (col.toArgb() == activeColor.toArgb()) Color.White else Color.White.copy(alpha = 0.2f),
                                            CircleShape
                                        )
                                        .clickable { onUpdateColor(activeColorTarget, col) }
                                )
                            }
                        }

                        Spacer(modifier = Modifier.height(12.dp))

                        // RGB Sliders
                        val currentRed = (activeColor.red * 255f).toInt()
                        val currentGreen = (activeColor.green * 255f).toInt()
                        val currentBlue = (activeColor.blue * 255f).toInt()

                        RgbSliderRow(
                            label = "R",
                            value = currentRed,
                            activeTrackColor = Color(0xFFFF4444),
                            onValueChange = { newR ->
                                val updated = Color(newR, currentGreen, currentBlue)
                                onUpdateColor(activeColorTarget, updated)
                            }
                        )

                        RgbSliderRow(
                            label = "G",
                            value = currentGreen,
                            activeTrackColor = Color(0xFF00E676),
                            onValueChange = { newG ->
                                val updated = Color(currentRed, newG, currentBlue)
                                onUpdateColor(activeColorTarget, updated)
                            }
                        )

                        RgbSliderRow(
                            label = "B",
                            value = currentBlue,
                            activeTrackColor = Color(0xFF00B4D8),
                            onValueChange = { newB ->
                                val updated = Color(currentRed, currentGreen, newB)
                                onUpdateColor(activeColorTarget, updated)
                            }
                        )
                    }
                }

                Spacer(modifier = Modifier.height(12.dp))

                // Bottom Done Button
                Button(
                    onClick = onDismiss,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(48.dp),
                    shape = RoundedCornerShape(12.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = NeonCyan,
                        contentColor = OnPrimary
                    )
                ) {
                    Text(
                        text = "Готово",
                        color = OnPrimary,
                        fontWeight = FontWeight.Bold,
                        fontSize = 14.sp
                    )
                }
            }
        }
    }
}

@Composable
private fun ThemeModeCard(
    title: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    isSelected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(12.dp))
            .background(if (isSelected) NeonCyan.copy(alpha = 0.15f) else ArcadeSurfaceVariant)
            .border(
                1.dp,
                if (isSelected) NeonCyan else ArcadeCardBorder,
                RoundedCornerShape(12.dp)
            )
            .clickable { onClick() }
            .padding(vertical = 12.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = if (isSelected) NeonCyan else TextSecondary,
            modifier = Modifier.size(20.dp)
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = title,
            color = if (isSelected) NeonCyan else TextPrimary,
            fontSize = 12.sp,
            fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium
        )
    }
}

@Composable
private fun TargetPill(
    label: String,
    color: Color,
    isSelected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier
            .clip(RoundedCornerShape(10.dp))
            .background(if (isSelected) NeonCyan.copy(alpha = 0.15f) else ArcadeSurfaceVariant)
            .border(
                1.dp,
                if (isSelected) NeonCyan else ArcadeCardBorder,
                RoundedCornerShape(10.dp)
            )
            .clickable { onClick() }
            .padding(vertical = 8.dp, horizontal = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Center
    ) {
        Box(
            modifier = Modifier
                .size(12.dp)
                .clip(CircleShape)
                .background(color)
                .border(1.dp, Color.White.copy(alpha = 0.3f), CircleShape)
        )
        Spacer(modifier = Modifier.width(6.dp))
        Text(
            text = label,
            color = if (isSelected) NeonCyan else TextSecondary,
            fontSize = 11.sp,
            fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
            maxLines = 1
        )
    }
}

@Composable
private fun RgbSliderRow(
    label: String,
    value: Int,
    activeTrackColor: Color,
    onValueChange: (Int) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = label,
            color = activeTrackColor,
            fontWeight = FontWeight.Bold,
            fontSize = 12.sp,
            modifier = Modifier.width(20.dp)
        )
        Slider(
            value = value.toFloat(),
            onValueChange = { onValueChange(it.toInt()) },
            valueRange = 0f..255f,
            colors = SliderDefaults.colors(
                thumbColor = Color.White,
                activeTrackColor = activeTrackColor,
                inactiveTrackColor = ArcadeSurfaceContainer
            ),
            modifier = Modifier.weight(1f)
        )
        Text(
            text = value.toString(),
            color = TextMuted,
            fontSize = 11.sp,
            modifier = Modifier.width(32.dp)
        )
    }
}
