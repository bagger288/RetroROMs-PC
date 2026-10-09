package com.example.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.ui.theme.GameBoyOlive
import com.example.ui.theme.GoldenAmber
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.NintendoRed
import com.example.ui.theme.PcEngineOrange
import com.example.ui.theme.RetroMagenta
import com.example.ui.theme.SegaBlue
import com.example.ui.theme.SonyTeal

@Composable
fun ConsolePlatformBadge(
    slug: String,
    name: String,
    modifier: Modifier = Modifier
) {
    val (badgeBg, badgeFg) = when (slug.lowercase()) {
        "dendy", "nes" -> NintendoRed to Color.White
        "genesis", "megadrive", "smd", "sms", "32x", "segacd" -> SegaBlue to Color.White
        "snes" -> Color(0xFF6A1B9A) to Color.White
        "gba" -> Color(0xFF4A148C) to NeonCyan
        "gb", "gbc" -> GameBoyOlive to Color.Black
        "psx", "ps1", "psp" -> SonyTeal to Color.White
        "n64" -> Color(0xFF00796B) to Color.White
        "dreamcast" -> Color(0xFFFF6F00) to Color.White
        "pce" -> PcEngineOrange to Color.White
        "atari2600" -> Color(0xFFD32F2F) to Color.White
        else -> MaterialTheme.colorScheme.surfaceVariant to NeonCyan
    }

    Box(
        modifier = modifier
            .clip(RoundedCornerShape(6.dp))
            .background(badgeBg.copy(alpha = 0.85f))
            .border(1.dp, badgeBg, RoundedCornerShape(6.dp))
            .padding(horizontal = 7.dp, vertical = 2.5.dp)
    ) {
        Text(
            text = name,
            color = badgeFg,
            fontSize = 11.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 0.5.sp
        )
    }
}

@Composable
fun RegionBadge(region: String, modifier: Modifier = Modifier) {
    val (color, label) = when (region.uppercase()) {
        "RU" -> RetroMagenta to "RU"
        "US" -> NeonCyan to "USA"
        "EU" -> GoldenAmber to "EUR"
        "JP" -> Color(0xFFFF5252) to "JPN"
        else -> Color.Gray to region
    }

    Box(
        modifier = modifier
            .clip(RoundedCornerShape(4.dp))
            .background(color.copy(alpha = 0.2f))
            .border(0.8.dp, color.copy(alpha = 0.7f), RoundedCornerShape(4.dp))
            .padding(horizontal = 5.dp, vertical = 1.5.dp)
    ) {
        Text(
            text = label,
            color = color,
            fontSize = 10.sp,
            fontWeight = FontWeight.SemiBold
        )
    }
}

@Composable
fun RatingBadge(rating: Float, modifier: Modifier = Modifier) {
    Row(
        modifier = modifier
            .clip(RoundedCornerShape(6.dp))
            .background(Color(0xFF1E2538))
            .padding(horizontal = 6.dp, vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = Icons.Default.Star,
            contentDescription = "Rating",
            tint = GoldenAmber,
            modifier = Modifier.size(13.dp)
        )
        Spacer(modifier = Modifier.width(3.dp))
        Text(
            text = String.format(java.util.Locale.US, "%.1f", rating),
            color = Color.White,
            fontSize = 11.sp,
            fontWeight = FontWeight.Bold
        )
    }
}
