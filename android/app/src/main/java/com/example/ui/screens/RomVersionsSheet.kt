package com.example.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.Translate
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.SheetState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.model.GameCard
import com.example.model.RomFileVersion
import com.example.ui.theme.ArcadeCardBorder
import com.example.ui.theme.ArcadeDarkBg
import com.example.ui.theme.ArcadeSurfaceContainer
import com.example.ui.theme.ArcadeSurfaceVariant
import com.example.ui.theme.GoldenAmber
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.OnPrimary
import com.example.ui.theme.RetroMagenta
import com.example.ui.theme.TextMuted
import com.example.ui.theme.TextPrimary
import com.example.ui.theme.TextSecondary

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RomVersionsSheet(
    game: GameCard,
    versions: List<RomFileVersion>,
    isLoading: Boolean,
    sheetState: SheetState,
    onSelectVersion: (GameCard, RomFileVersion) -> Unit,
    onDismiss: () -> Unit
) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = ArcadeDarkBg,
        tonalElevation = 8.dp,
        modifier = Modifier.testTag("rom_versions_sheet")
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 12.dp)
        ) {
            // Header
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Выбор версии ROM",
                        color = NeonCyan,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Bold,
                        letterSpacing = 0.5.sp
                    )
                    Spacer(modifier = Modifier.height(2.dp))
                    Text(
                        text = game.title,
                        color = TextPrimary,
                        fontSize = 17.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }

                IconButton(
                    onClick = onDismiss,
                    modifier = Modifier
                        .size(36.dp)
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

            if (isLoading) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(180.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        CircularProgressIndicator(
                            color = NeonCyan,
                            modifier = Modifier.size(32.dp),
                            strokeWidth = 2.5.dp
                        )
                        Spacer(modifier = Modifier.height(12.dp))
                        Text(
                            text = "Поиск доступных версий игры на Emu-Land…",
                            color = TextSecondary,
                            fontSize = 13.sp
                        )
                    }
                }
            } else if (versions.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(140.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = "Версии не найдены, доступна стандартная загрузка",
                        color = TextMuted,
                        fontSize = 13.sp
                    )
                }
            } else {
                Text(
                    text = "ДОСТУПНО ВЕРСИЙ ДЛЯ СКАЧИВАНИЯ (${versions.size})",
                    color = TextMuted,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 0.5.sp
                )
                Spacer(modifier = Modifier.height(8.dp))

                // Group versions by category
                val grouped = versions.groupBy { it.category }

                LazyColumn(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(340.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    grouped.forEach { (catName, itemsInCat) ->
                        val cleanCatTitle = catName
                            .replace(Regex("\\s*\\(\\?\\)\\s*"), "")
                            .trim()
                            .let { raw ->
                                val words = raw.split(" ").filter { it.isNotBlank() }
                                if (words.size >= 2 && words[0].equals(words[1], ignoreCase = true)) {
                                    words.drop(1).joinToString(" ")
                                } else {
                                    raw
                                }
                            }
                            .uppercase()

                        item {
                            Text(
                                text = cleanCatTitle,
                                color = GoldenAmber,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(top = 8.dp, bottom = 4.dp)
                            )
                        }

                        items(itemsInCat, key = { it.fid + it.name }) { version ->
                            val isRussian = version.regionOrType == "RUS"

                            Box(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clip(RoundedCornerShape(12.dp))
                                    .background(if (isRussian) ArcadeSurfaceContainer else ArcadeSurfaceVariant)
                                    .border(
                                        1.dp,
                                        if (isRussian) NeonCyan.copy(alpha = 0.6f) else ArcadeCardBorder,
                                        RoundedCornerShape(12.dp)
                                    )
                                    .clickable { onSelectVersion(game, version) }
                                    .padding(horizontal = 14.dp, vertical = 12.dp)
                            ) {
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.SpaceBetween
                                ) {
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically,
                                        modifier = Modifier.weight(1f)
                                    ) {
                                        // Region Badge
                                        Box(
                                            modifier = Modifier
                                                .clip(RoundedCornerShape(6.dp))
                                                .background(
                                                    when (version.regionOrType) {
                                                        "RUS" -> RetroMagenta.copy(alpha = 0.2f)
                                                        "USA" -> NeonCyan.copy(alpha = 0.2f)
                                                        "EUR" -> GoldenAmber.copy(alpha = 0.2f)
                                                        "JAP" -> Color(0xFFFF5252).copy(alpha = 0.2f)
                                                        "PIRATE" -> Color(0xFFFF9100).copy(alpha = 0.2f)
                                                        "GoodNES" -> NeonCyan.copy(alpha = 0.15f)
                                                        else -> ArcadeSurfaceContainer
                                                    }
                                                )
                                                .padding(horizontal = 6.dp, vertical = 3.dp)
                                        ) {
                                            Text(
                                                text = when (version.regionOrType) {
                                                    "RUS" -> "🇷🇺 RUS"
                                                    "USA" -> "🇺🇸 USA"
                                                    "EUR" -> "🇪🇺 EUR"
                                                    "JAP" -> "🇯🇵 JAP"
                                                    "WLD" -> "🌐 WORLD"
                                                    "BETA" -> "⚙️ BETA"
                                                    "HACK" -> "★ HACK"
                                                    "PIRATE" -> "☠ PIRATE"
                                                    "GoodNES" -> "📦 GoodNES"
                                                    else -> "ROM"
                                                },
                                                color = when (version.regionOrType) {
                                                    "RUS" -> RetroMagenta
                                                    "USA" -> NeonCyan
                                                    "EUR" -> GoldenAmber
                                                    "JAP" -> Color(0xFFFF5252)
                                                    "PIRATE" -> Color(0xFFFF9100)
                                                    "GoodNES" -> NeonCyan
                                                    else -> TextSecondary
                                                },
                                                fontSize = 11.sp,
                                                fontWeight = FontWeight.Bold
                                            )
                                        }

                                        Spacer(modifier = Modifier.width(10.dp))

                                        Column {
                                            Text(
                                                text = version.name,
                                                color = TextPrimary,
                                                fontSize = 13.sp,
                                                fontWeight = FontWeight.SemiBold,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis
                                            )
                                            Spacer(modifier = Modifier.height(2.dp))
                                            Text(
                                                text = "Размер: ${version.size}",
                                                color = TextMuted,
                                                fontSize = 11.sp
                                            )
                                        }
                                    }

                                    Spacer(modifier = Modifier.width(8.dp))

                                    Box(
                                        modifier = Modifier
                                            .size(36.dp)
                                            .clip(RoundedCornerShape(8.dp))
                                            .background(NeonCyan),
                                        contentAlignment = Alignment.Center
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.Download,
                                            contentDescription = "Скачать",
                                            tint = OnPrimary,
                                            modifier = Modifier.size(18.dp)
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Spacer(modifier = Modifier.height(16.dp))
        }
    }
}
