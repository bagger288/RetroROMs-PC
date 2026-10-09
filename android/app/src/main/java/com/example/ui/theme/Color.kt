package com.example.ui.theme

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// Top-level dynamic colors bound to current app theme
val ArcadeDarkBg: Color
    @Composable
    get() = LocalAppColors.current.background

val ArcadeSurface: Color
    @Composable
    get() = LocalAppColors.current.surface

val ArcadeSurfaceVariant: Color
    @Composable
    get() = LocalAppColors.current.surfaceVariant

val ArcadeSurfaceContainer: Color
    @Composable
    get() = LocalAppColors.current.surfaceContainer

val ArcadeCardBorder: Color
    @Composable
    get() = LocalAppColors.current.cardBorder

val NeonCyan: Color
    @Composable
    get() = LocalAppColors.current.primaryAccent

val OnPrimary: Color
    @Composable
    get() = LocalAppColors.current.onPrimaryAccent

val NeonCyanVariant: Color
    @Composable
    get() = LocalAppColors.current.primaryAccent.copy(alpha = 0.8f)

val GoldenAmber: Color
    @Composable
    get() = LocalAppColors.current.secondaryAccent

val RetroMagenta: Color
    @Composable
    get() = LocalAppColors.current.tertiaryAccent

val EmeraldGreen = Color(0xFF00E676)

val TextPrimary: Color
    @Composable
    get() = LocalAppColors.current.textPrimary

val TextSecondary: Color
    @Composable
    get() = LocalAppColors.current.textSecondary

val TextMuted: Color
    @Composable
    get() = LocalAppColors.current.textMuted

// Console brand badges (static accent references)
val NintendoRed = Color(0xFFE60012)
val SegaBlue = Color(0xFF0089CF)
val SonyTeal = Color(0xFF003791)
val GameBoyOlive = Color(0xFF8BAC0F)
val AtariRed = Color(0xFFE41E26)
val PcEngineOrange = Color(0xFFFF6600)
