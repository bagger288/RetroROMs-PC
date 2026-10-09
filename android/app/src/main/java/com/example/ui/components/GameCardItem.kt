package com.example.ui.components

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.basicMarquee
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.FavoriteBorder
import androidx.compose.material.icons.filled.Gamepad
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import com.example.model.GameCard
import com.example.ui.theme.ArcadeCardBorder
import com.example.ui.theme.ArcadeSurfaceVariant
import com.example.ui.theme.EmeraldGreen
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.OnPrimary
import com.example.ui.theme.RetroMagenta
import com.example.ui.theme.TextPrimary
import com.example.ui.theme.TextSecondary
import com.example.ui.viewmodel.DownloadProgressState

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun GameCardItem(
    game: GameCard,
    downloadProgress: DownloadProgressState?,
    onCardClick: () -> Unit,
    onLongClick: () -> Unit = {},
    onDownloadClick: () -> Unit,
    onFavoriteClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val isDownloading = downloadProgress != null
    val cardShape = RoundedCornerShape(12.dp)
    val haptic = LocalHapticFeedback.current

    val interactionSource = remember { MutableInteractionSource() }
    val isPressed by interactionSource.collectIsPressedAsState()
    var isInteracted by remember { mutableStateOf(false) }

    LaunchedEffect(isPressed) {
        if (isPressed) {
            isInteracted = true
        }
    }

    val platformShortName = when (game.consoleSlug.lowercase()) {
        "dendy" -> "NES"
        "genesis" -> "Genesis"
        "snes" -> "SNES"
        "gba" -> "GBA"
        "gb" -> "Game Boy"
        "gbc" -> "GBC"
        "psx" -> "PS1"
        "n64" -> "N64"
        "dreamcast" -> "Dreamcast"
        "saturn" -> "Saturn"
        "nds" -> "NDS"
        "psp" -> "PSP"
        "sms" -> "Master System"
        "pce" -> "PC Engine"
        "2600" -> "Atari 2600"
        "3do" -> "3DO"
        "32x" -> "Sega 32X"
        "segacd" -> "Sega CD"
        "gg" -> "Game Gear"
        else -> game.consoleName.split("/").first().trim()
    }

    Box(
        modifier = modifier
            .testTag("game_item_card_${game.id}")
            .clip(cardShape)
            .background(ArcadeSurfaceVariant)
            .border(
                1.dp,
                if (isDownloading) NeonCyan else ArcadeCardBorder,
                cardShape
            )
            .combinedClickable(
                interactionSource = interactionSource,
                indication = null,
                onClick = {
                    isInteracted = true
                    onCardClick()
                },
                onLongClick = {
                    haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                    isInteracted = true
                    onLongClick()
                }
            )
    ) {
        Column(modifier = Modifier.fillMaxWidth()) {
            // Header Image Box with neat aspect ratio
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(16f / 10f)
                    .background(Color(0xFF0F141E))
            ) {
                if (!game.coverUrl.isNullOrEmpty()) {
                    AsyncImage(
                        model = game.coverUrl,
                        contentDescription = "Cover for ${game.title}",
                        contentScale = ContentScale.Crop,
                        modifier = Modifier.fillMaxSize()
                    )
                } else {
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(
                                Brush.radialGradient(
                                    listOf(Color(0xFF1E283C), Color(0xFF0F141E))
                                )
                            ),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            imageVector = Icons.Default.Gamepad,
                            contentDescription = null,
                            tint = NeonCyan.copy(alpha = 0.4f),
                            modifier = Modifier.size(36.dp)
                        )
                    }
                }

                // Top badges overlay
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(6.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    RatingBadge(rating = game.rating)

                    IconButton(
                        onClick = onFavoriteClick,
                        modifier = Modifier
                            .size(28.dp)
                            .testTag("favorite_btn_${game.id}")
                            .clip(RoundedCornerShape(6.dp))
                            .background(Color.Black.copy(alpha = 0.55f))
                    ) {
                        Icon(
                            imageVector = if (game.isFavorite) Icons.Default.Favorite else Icons.Default.FavoriteBorder,
                            contentDescription = if (game.isFavorite) "Remove from favorites" else "Add to favorites",
                            tint = if (game.isFavorite) RetroMagenta else Color.White,
                            modifier = Modifier.size(15.dp)
                        )
                    }
                }

                // Bottom gradient scrim
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(20.dp)
                        .align(Alignment.BottomCenter)
                        .background(
                            Brush.verticalGradient(
                                listOf(Color.Transparent, ArcadeSurfaceVariant)
                            )
                        )
                )
            }

            // Compact Card Body Info (~78dp, eliminating wasted space)
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 8.dp, vertical = 8.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                // Title (Single line). When touched/interacted, scrolls as marquee ("бегущая строка")
                val isMarquee = isPressed || isInteracted
                Text(
                    text = game.title,
                    color = TextPrimary,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center,
                    maxLines = 1,
                    overflow = if (isMarquee) TextOverflow.Clip else TextOverflow.Ellipsis,
                    modifier = Modifier
                        .fillMaxWidth()
                        .then(
                            if (isMarquee) {
                                Modifier.basicMarquee(
                                    iterations = 4,
                                    initialDelayMillis = 300,
                                    repeatDelayMillis = 800,
                                    velocity = 32.dp
                                )
                            } else Modifier
                        )
                        .clickable(
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null
                        ) {
                            isInteracted = !isInteracted
                        }
                )

                Spacer(modifier = Modifier.height(2.dp))

                // Subtitle (Genre • Year)
                val subtitle = if (game.year.isNotBlank() && game.genre.isNotBlank()) {
                    "${game.genre} • ${game.year}"
                } else if (game.genre.isNotBlank()) {
                    game.genre
                } else {
                    game.year
                }

                Text(
                    text = subtitle.ifBlank { "ROM" },
                    color = TextSecondary,
                    fontSize = 11.sp,
                    textAlign = TextAlign.Center,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.fillMaxWidth()
                )

                Spacer(modifier = Modifier.height(6.dp))

                // Bottom action row: Platform label + Compact Download button (32dp), centered
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = platformShortName,
                        color = NeonCyan,
                        fontSize = 10.5.sp,
                        fontWeight = FontWeight.Bold,
                        textAlign = TextAlign.Center,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier
                            .weight(1f, fill = false)
                            .clip(RoundedCornerShape(6.dp))
                            .background(NeonCyan.copy(alpha = 0.12f))
                            .padding(horizontal = 7.dp, vertical = 4.dp)
                    )

                    Spacer(modifier = Modifier.width(6.dp))

                    // Compact Download button (32x32dp)
                    Box(
                        modifier = Modifier
                            .testTag("download_btn_${game.id}")
                            .size(32.dp)
                            .clip(RoundedCornerShape(8.dp))
                            .background(
                                when {
                                    game.isDownloaded -> EmeraldGreen.copy(alpha = 0.2f)
                                    isDownloading -> NeonCyan.copy(alpha = 0.2f)
                                    else -> NeonCyan
                                }
                            )
                            .clickable(enabled = !isDownloading) { onDownloadClick() },
                        contentAlignment = Alignment.Center
                    ) {
                        when {
                            isDownloading -> {
                                CircularProgressIndicator(
                                    progress = { (downloadProgress.percent / 100f).coerceIn(0f, 1f) },
                                    modifier = Modifier.size(16.dp),
                                    color = NeonCyan,
                                    strokeWidth = 2.dp
                                )
                            }
                            game.isDownloaded -> {
                                Icon(
                                    imageVector = Icons.Default.CheckCircle,
                                    contentDescription = "Скачано",
                                    tint = EmeraldGreen,
                                    modifier = Modifier.size(18.dp)
                                )
                            }
                            else -> {
                                Icon(
                                    imageVector = Icons.Default.Download,
                                    contentDescription = "Скачать ROM",
                                    tint = OnPrimary,
                                    modifier = Modifier.size(16.dp)
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
