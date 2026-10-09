package com.example.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.graphics.Color

@Composable
fun MyApplicationTheme(
    appColors: AppThemeColors = DefaultDarkTheme,
    content: @Composable () -> Unit
) {
    val materialColorScheme = if (appColors.isLight) {
        lightColorScheme(
            primary = appColors.primaryAccent,
            onPrimary = appColors.onPrimaryAccent,
            primaryContainer = appColors.primaryAccent.copy(alpha = 0.15f),
            onPrimaryContainer = appColors.primaryAccent,
            secondary = appColors.secondaryAccent,
            onSecondary = Color.White,
            tertiary = appColors.tertiaryAccent,
            onTertiary = Color.White,
            background = appColors.background,
            onBackground = appColors.textPrimary,
            surface = appColors.surface,
            onSurface = appColors.textPrimary,
            surfaceVariant = appColors.surfaceVariant,
            onSurfaceVariant = appColors.textSecondary,
            surfaceContainer = appColors.surfaceContainer,
            outline = appColors.cardBorder
        )
    } else {
        darkColorScheme(
            primary = appColors.primaryAccent,
            onPrimary = appColors.onPrimaryAccent,
            primaryContainer = appColors.primaryAccent.copy(alpha = 0.2f),
            onPrimaryContainer = appColors.primaryAccent,
            secondary = appColors.secondaryAccent,
            onSecondary = Color.Black,
            tertiary = appColors.tertiaryAccent,
            onTertiary = Color.White,
            background = appColors.background,
            onBackground = appColors.textPrimary,
            surface = appColors.surface,
            onSurface = appColors.textPrimary,
            surfaceVariant = appColors.surfaceVariant,
            onSurfaceVariant = appColors.textSecondary,
            surfaceContainer = appColors.surfaceContainer,
            outline = appColors.cardBorder
        )
    }

    CompositionLocalProvider(LocalAppColors provides appColors) {
        MaterialTheme(
            colorScheme = materialColorScheme,
            typography = Typography,
            content = content
        )
    }
}
