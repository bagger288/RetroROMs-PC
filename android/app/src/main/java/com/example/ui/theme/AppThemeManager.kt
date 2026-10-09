package com.example.ui.theme

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb

enum class ThemeMode {
    DARK,
    LIGHT,
    CUSTOM
}

enum class CatalogViewMode {
    GRID,
    LIST
}

data class AppThemeColors(
    val primaryAccent: Color,
    val secondaryAccent: Color,
    val tertiaryAccent: Color,
    val background: Color,
    val surface: Color,
    val surfaceVariant: Color,
    val surfaceContainer: Color,
    val cardBorder: Color,
    val textPrimary: Color,
    val textSecondary: Color,
    val textMuted: Color,
    val onPrimaryAccent: Color,
    val isLight: Boolean
)

val DefaultDarkTheme = AppThemeColors(
    primaryAccent = Color(0xFF00E5FF),
    secondaryAccent = Color(0xFFFFB300),
    tertiaryAccent = Color(0xFFFF3366),
    background = Color(0xFF0B0E14),
    surface = Color(0xFF141923),
    surfaceVariant = Color(0xFF1E2535),
    surfaceContainer = Color(0xFF232B3E),
    cardBorder = Color(0xFF2F3B52),
    textPrimary = Color(0xFFF1F5F9),
    textSecondary = Color(0xFF94A3B8),
    textMuted = Color(0xFF64748B),
    onPrimaryAccent = Color(0xFF000000),
    isLight = false
)

val DefaultLightTheme = AppThemeColors(
    primaryAccent = Color(0xFF0284C7),
    secondaryAccent = Color(0xFFD97706),
    tertiaryAccent = Color(0xFFE11D48),
    background = Color(0xFFF6F8FC),
    surface = Color(0xFFFFFFFF),
    surfaceVariant = Color(0xFFEAEFF5),
    surfaceContainer = Color(0xFFDFE6F0),
    cardBorder = Color(0xFFCBD5E1),
    textPrimary = Color(0xFF0F172A),
    textSecondary = Color(0xFF334155),
    textMuted = Color(0xFF64748B),
    onPrimaryAccent = Color(0xFFFFFFFF),
    isLight = true
)

val LocalAppColors = compositionLocalOf { DefaultDarkTheme }

object AppTheme {
    val colors: AppThemeColors
        @Composable
        get() = LocalAppColors.current
}

data class ThemePreset(
    val name: String,
    val primary: Color,
    val secondary: Color,
    val background: Color,
    val surface: Color,
    val isLight: Boolean,
    val onPrimary: Color = if (isLight) Color.White else Color.Black
)

val PredefinedPresets = listOf(
    ThemePreset("Киберпанк", Color(0xFF00E5FF), Color(0xFFFFB300), Color(0xFF0B0E14), Color(0xFF141923), false, Color(0xFF000000)),
    ThemePreset("Светлая классика", Color(0xFF0284C7), Color(0xFFD97706), Color(0xFFF6F8FC), Color(0xFFFFFFFF), true, Color(0xFFFFFFFF)),
    ThemePreset("Nintendo Retro", Color(0xFFE60012), Color(0xFFFFCC00), Color(0xFF14141E), Color(0xFF20202E), false, Color(0xFFFFFFFF)),
    ThemePreset("Sega Mega", Color(0xFF0089CF), Color(0xFFFFA000), Color(0xFF0D131F), Color(0xFF182236), false, Color(0xFFFFFFFF)),
    ThemePreset("Game Boy Lime", Color(0xFF8BAC0F), Color(0xFF9BBC0F), Color(0xFF121B10), Color(0xFF1C2A18), false, Color(0xFF000000)),
    ThemePreset("Synthwave 80s", Color(0xFFC026D3), Color(0xFF06B6D4), Color(0xFF120B1E), Color(0xFF1E1333), false, Color(0xFFFFFFFF)),
    ThemePreset("OLED Pure Black", Color(0xFF00E676), Color(0xFFFFB300), Color(0xFF000000), Color(0xFF121212), false, Color(0xFF000000))
)

class ThemePreferences(context: Context) {
    private val prefs = context.getSharedPreferences("retroms_theme_prefs", Context.MODE_PRIVATE)

    fun getThemeMode(): ThemeMode {
        val name = prefs.getString("theme_mode", ThemeMode.DARK.name) ?: ThemeMode.DARK.name
        return try {
            ThemeMode.valueOf(name)
        } catch (_: Exception) {
            ThemeMode.DARK
        }
    }

    fun setThemeMode(mode: ThemeMode) {
        prefs.edit().putString("theme_mode", mode.name).apply()
    }

    fun getCustomPrimary(): Color {
        val argb = prefs.getInt("custom_primary", DefaultDarkTheme.primaryAccent.toArgb().toInt())
        return Color(argb)
    }

    fun setCustomPrimary(color: Color) {
        prefs.edit().putInt("custom_primary", color.toArgb().toInt()).apply()
    }

    fun getCustomBackground(): Color {
        val argb = prefs.getInt("custom_bg", DefaultDarkTheme.background.toArgb().toInt())
        return Color(argb)
    }

    fun setCustomBackground(color: Color) {
        prefs.edit().putInt("custom_bg", color.toArgb().toInt()).apply()
    }

    fun getCustomSurface(): Color {
        val argb = prefs.getInt("custom_surface", DefaultDarkTheme.surface.toArgb().toInt())
        return Color(argb)
    }

    fun setCustomSurface(color: Color) {
        prefs.edit().putInt("custom_surface", color.toArgb().toInt()).apply()
    }

    fun getCustomSecondary(): Color {
        val argb = prefs.getInt("custom_secondary", DefaultDarkTheme.secondaryAccent.toArgb().toInt())
        return Color(argb)
    }

    fun setCustomSecondary(color: Color) {
        prefs.edit().putInt("custom_secondary", color.toArgb().toInt()).apply()
    }

    fun getCustomOnPrimary(): Color {
        val argb = prefs.getInt("custom_on_primary", DefaultDarkTheme.onPrimaryAccent.toArgb().toInt())
        return Color(argb)
    }

    fun setCustomOnPrimary(color: Color) {
        prefs.edit().putInt("custom_on_primary", color.toArgb().toInt()).apply()
    }

    fun getCatalogViewMode(): CatalogViewMode {
        val name = prefs.getString("catalog_view_mode", CatalogViewMode.GRID.name) ?: CatalogViewMode.GRID.name
        return try {
            CatalogViewMode.valueOf(name)
        } catch (_: Exception) {
            CatalogViewMode.GRID
        }
    }

    fun setCatalogViewMode(mode: CatalogViewMode) {
        prefs.edit().putString("catalog_view_mode", mode.name).apply()
    }

    fun resetToDefaults() {
        prefs.edit().clear().apply()
    }
}
