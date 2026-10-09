package com.example.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.Gamepad
import androidx.compose.material.icons.filled.Language
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.SheetState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import com.example.model.GameCard
import com.example.ui.components.ConsolePlatformBadge
import com.example.ui.components.RegionBadge
import com.example.ui.theme.ArcadeCardBorder
import com.example.ui.theme.ArcadeSurface
import com.example.ui.theme.ArcadeSurfaceContainer
import com.example.ui.theme.ArcadeSurfaceVariant
import com.example.ui.theme.EmeraldGreen
import com.example.ui.theme.GoldenAmber
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.OnPrimary
import com.example.ui.theme.RetroMagenta
import com.example.ui.theme.TextMuted
import com.example.ui.theme.TextPrimary
import com.example.ui.theme.TextSecondary
import com.example.ui.viewmodel.DownloadProgressState

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GameDetailSheet(
    game: GameCard,
    targetFolderName: String,
    downloadProgress: DownloadProgressState?,
    isLoadingDetail: Boolean,
    sheetState: SheetState,
    onDismiss: () -> Unit,
    onDownloadClick: () -> Unit
) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = ArcadeSurface,
        scrimColor = Color.Black.copy(alpha = 0.75f),
        dragHandle = null
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
                .padding(bottom = 32.dp)
        ) {
            // Header Bar
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp, vertical = 14.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                ConsolePlatformBadge(
                    slug = game.consoleSlug,
                    name = game.consoleName
                )

                IconButton(
                    onClick = onDismiss,
                    modifier = Modifier
                        .size(36.dp)
                        .clip(RoundedCornerShape(10.dp))
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

            // Title and Subtitle
            Column(modifier = Modifier.padding(horizontal = 20.dp)) {
                Text(
                    text = game.title,
                    color = TextPrimary,
                    fontSize = 22.sp,
                    fontWeight = FontWeight.ExtraBold,
                    lineHeight = 28.sp
                )

                if (!game.originalTitle.isNullOrEmpty()) {
                    Spacer(modifier = Modifier.height(2.dp))
                    Text(
                        text = game.originalTitle,
                        color = NeonCyan.copy(alpha = 0.8f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Medium
                    )
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Screenshots Gallery
            val allScreens = if (game.screenshotUrls.isNotEmpty()) {
                game.screenshotUrls
            } else if (!game.coverUrl.isNullOrEmpty()) {
                listOf(game.coverUrl)
            } else {
                emptyList()
            }

            if (allScreens.isNotEmpty()) {
                LazyRow(
                    modifier = Modifier.fillMaxWidth(),
                    contentPadding = PaddingValues(horizontal = 20.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(allScreens) { screenUrl ->
                        Box(
                            modifier = Modifier
                                .width(280.dp)
                                .aspectRatio(16f / 10f)
                                .clip(RoundedCornerShape(12.dp))
                                .border(1.dp, ArcadeCardBorder, RoundedCornerShape(12.dp))
                                .background(Color(0xFF0F141E))
                        ) {
                            AsyncImage(
                                model = screenUrl,
                                contentDescription = "Gameplay screenshot",
                                contentScale = ContentScale.Crop,
                                modifier = Modifier.matchParentSize()
                            )
                        }
                    }
                }
            } else {
                // Placeholder banner
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 20.dp)
                        .height(160.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .background(ArcadeSurfaceContainer)
                        .border(1.dp, ArcadeCardBorder, RoundedCornerShape(12.dp)),
                    contentAlignment = Alignment.Center
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Icon(
                            imageVector = Icons.Default.Gamepad,
                            contentDescription = null,
                            tint = NeonCyan.copy(alpha = 0.5f),
                            modifier = Modifier.size(48.dp)
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "Screenshots from Emu-Land",
                            color = TextMuted,
                            fontSize = 12.sp
                        )
                    }
                }
            }

            if (isLoadingDetail) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 10.dp, end = 20.dp),
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(14.dp),
                        strokeWidth = 2.dp,
                        color = NeonCyan
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = "Loading extra media…",
                        color = TextMuted,
                        fontSize = 11.sp
                    )
                }
            }

            Spacer(modifier = Modifier.height(18.dp))

            // Tech Specs Grid (Year, Genre, Publisher, File Size, Rating)
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp)
                    .clip(RoundedCornerShape(14.dp))
                    .background(ArcadeSurfaceVariant)
                    .border(1.dp, ArcadeCardBorder, RoundedCornerShape(14.dp))
                    .padding(14.dp)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    SpecColumn(label = "GENRE", value = game.genre)
                    SpecColumn(label = "RELEASE YEAR", value = game.year)
                    SpecColumn(label = "FILE SIZE", value = game.fileSize, valueColor = NeonCyan)
                }

                HorizontalDivider(
                    modifier = Modifier.padding(vertical = 10.dp),
                    color = ArcadeCardBorder
                )

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    SpecColumn(label = "PUBLISHER", value = game.publisher)
                    SpecColumn(label = "DEVELOPER", value = game.developer)
                    SpecColumn(label = "RATING", value = "★ ${game.rating}", valueColor = GoldenAmber)
                }
            }

            Spacer(modifier = Modifier.height(18.dp))

            // Description Section
            Column(modifier = Modifier.padding(horizontal = 20.dp)) {
                Text(
                    text = "DESCRIPTION / ОПИСАНИЕ",
                    color = TextMuted,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 1.sp
                )

                Spacer(modifier = Modifier.height(8.dp))

                Text(
                    text = if (game.description.isNotBlank()) game.description else "Описание игры подгружается с Emu-Land.net. Вы можете сразу скачать ROM для запуска в вашем любимом эмуляторе.",
                    color = TextSecondary,
                    fontSize = 14.sp,
                    lineHeight = 22.sp
                )
            }

            Spacer(modifier = Modifier.height(24.dp))

            // Download Progress & Button
            val isDownloading = downloadProgress != null

            Column(modifier = Modifier.padding(horizontal = 20.dp)) {
                if (isDownloading) {
                    LinearProgressIndicator(
                        progress = { (downloadProgress.percent / 100f).coerceIn(0f, 1f) },
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(8.dp)
                            .clip(RoundedCornerShape(4.dp)),
                        color = NeonCyan,
                        trackColor = ArcadeCardBorder
                    )
                    Spacer(modifier = Modifier.height(6.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(
                            text = "Downloading… ${downloadProgress.percent}%",
                            color = NeonCyan,
                            fontSize = 12.sp,
                            fontWeight = FontWeight.SemiBold
                        )
                        Text(
                            text = "Saving to $targetFolderName",
                            color = TextMuted,
                            fontSize = 11.sp
                        )
                    }
                    Spacer(modifier = Modifier.height(12.dp))
                }

                Button(
                    onClick = onDownloadClick,
                    enabled = !isDownloading,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(54.dp)
                        .testTag("modal_download_button"),
                    shape = RoundedCornerShape(14.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = if (game.isDownloaded) EmeraldGreen else NeonCyan,
                        contentColor = OnPrimary
                    )
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            imageVector = if (game.isDownloaded) Icons.Default.CheckCircle else Icons.Default.Download,
                            contentDescription = null,
                            modifier = Modifier.size(22.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = when {
                                isDownloading -> "Загрузка (${downloadProgress.percent}%)"
                                game.isDownloaded -> "Скачано (нажмите для повторной загрузки)"
                                else -> "Выбрать версию и скачать"
                            },
                            fontWeight = FontWeight.Bold,
                            fontSize = 14.sp,
                            letterSpacing = 0.5.sp
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun SpecColumn(
    label: String,
    value: String,
    valueColor: Color? = null
) {
    val finalColor = valueColor ?: TextPrimary
    Column {
        Text(
            text = label,
            color = TextMuted,
            fontSize = 10.sp,
            fontWeight = FontWeight.SemiBold,
            letterSpacing = 0.5.sp
        )
        Spacer(modifier = Modifier.height(2.dp))
        Text(
            text = value,
            color = finalColor,
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold
        )
    }
}
