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
import androidx.compose.foundation.layout.fillMaxHeight
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
import androidx.compose.material.icons.outlined.FavoriteBorder
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
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import com.example.R
import com.example.model.GameCard
import com.example.ui.theme.ArcadeCardBorder
import com.example.ui.theme.ArcadeSurfaceContainer
import com.example.ui.theme.ArcadeSurfaceVariant
import com.example.ui.theme.EmeraldGreen
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.NintendoRed
import com.example.ui.theme.RetroMagenta
import com.example.ui.theme.SegaBlue
import com.example.ui.theme.SonyTeal
import com.example.ui.theme.TextMuted
import com.example.ui.theme.TextPrimary
import com.example.ui.theme.TextSecondary
import com.example.ui.viewmodel.DownloadProgressState

/**
 * Вариант 4: «Инфо-таблица / Список-реестр» (Финальный)
 * - Строго фиксированная высота 48 dp: каждый тайл одинакового размера без скачков.
 * - Без рейтинга (как в изначальном виде).
 * - Удержание пальца (Long Click) на обложке открывает скриншот во весь экран.
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun GameListItem(
    game: GameCard,
    downloadProgress: DownloadProgressState?,
    onCardClick: () -> Unit,
    onLongClick: () -> Unit = {},
    onCoverLongClick: () -> Unit = onLongClick,
    onDownloadClick: () -> Unit,
    onFavoriteClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val isDownloading = downloadProgress != null
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
        "genesis" -> "MD"
        "snes" -> "SNES"
        "gba" -> "GBA"
        "gb" -> "GB"
        "gbc" -> "GBC"
        "psx" -> "PS1"
        "n64" -> "N64"
        "dreamcast" -> "DC"
        "saturn" -> "SS"
        "nds" -> "NDS"
        "psp" -> "PSP"
        "sms" -> "SMS"
        "pce" -> "PCE"
        "2600" -> "2600"
        "3do" -> "3DO"
        "32x" -> "32X"
        "segacd" -> "SCD"
        "gg" -> "GG"
        else -> game.consoleName.split("/").first().trim()
    }

    val brandColor = when (game.consoleSlug.lowercase()) {
        "dendy", "snes", "gba", "gb", "gbc", "n64", "nds" -> NintendoRed
        "genesis", "dreamcast", "saturn", "sms", "32x", "segacd", "gg" -> SegaBlue
        "psx", "psp" -> SonyTeal
        "pce" -> NeonCyan
        "3do", "2600" -> RetroMagenta
        else -> NeonCyan
    }

    val itemShape = RoundedCornerShape(8.dp)

    Box(
        modifier = modifier
            .testTag("game_item_${game.id}")
            .fillMaxWidth()
            .height(52.dp)
            .clip(itemShape)
            .background(ArcadeSurfaceVariant.copy(alpha = 0.65f))
            .border(
                0.8.dp,
                if (isDownloading) NeonCyan else ArcadeCardBorder.copy(alpha = 0.8f),
                itemShape
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
            .padding(horizontal = 8.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxSize(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Game Cover / Screenshot Thumbnail
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(RoundedCornerShape(6.dp))
                    .background(ArcadeSurfaceContainer)
                    .border(0.5.dp, ArcadeCardBorder, RoundedCornerShape(6.dp)),
                contentAlignment = Alignment.Center
            ) {
                if (!game.coverUrl.isNullOrEmpty()) {
                    AsyncImage(
                        model = game.coverUrl,
                        contentDescription = game.title,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier.fillMaxSize()
                    )
                } else {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_game_cartridge_outlined),
                        contentDescription = null,
                        tint = brandColor.copy(alpha = 0.7f),
                        modifier = Modifier.size(18.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.width(8.dp))

            // Main Info Column: Vertically centered relative to the tile and cover image
            Column(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight(),
                verticalArrangement = Arrangement.Center
            ) {
                // Line 1: Game Title + Year
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    val isMarquee = isPressed || isInteracted
                    Text(
                        text = game.title,
                        color = TextPrimary,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        overflow = if (isMarquee) TextOverflow.Clip else TextOverflow.Ellipsis,
                        modifier = Modifier
                            .weight(1f, fill = false)
                            .then(
                                if (isMarquee) {
                                    Modifier.basicMarquee(
                                        iterations = 4,
                                        initialDelayMillis = 300,
                                        repeatDelayMillis = 800,
                                        velocity = 30.dp
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
                    if (game.year.isNotBlank() && game.year != "0") {
                        Spacer(modifier = Modifier.width(6.dp))
                        Text(
                            text = "(${game.year})",
                            color = TextMuted,
                            fontSize = 11.sp,
                            maxLines = 1
                        )
                    }
                }

                Spacer(modifier = Modifier.height(2.dp))

                // Line 2: Platform + Genre (No rating, ample vertical space for descenders)
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(
                        text = platformShortName,
                        color = brandColor,
                        fontSize = 10.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1
                    )

                    if (game.genre.isNotBlank()) {
                        Text(
                            text = "  •  ${game.genre}",
                            color = TextSecondary,
                            fontSize = 10.5.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.width(4.dp))

            // Action Buttons
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(1.dp)
            ) {
                IconButton(
                    onClick = onFavoriteClick,
                    modifier = Modifier.size(28.dp)
                ) {
                    Icon(
                        imageVector = if (game.isFavorite) Icons.Default.Favorite else Icons.Outlined.FavoriteBorder,
                        contentDescription = "В избранное",
                        tint = if (game.isFavorite) RetroMagenta else TextMuted,
                        modifier = Modifier.size(15.dp)
                    )
                }

                IconButton(
                    onClick = onDownloadClick,
                    enabled = !isDownloading,
                    modifier = Modifier.size(30.dp)
                ) {
                    if (isDownloading) {
                        CircularProgressIndicator(
                            progress = { (downloadProgress?.percent ?: 0) / 100f },
                            modifier = Modifier.size(14.dp),
                            color = NeonCyan,
                            strokeWidth = 2.dp
                        )
                    } else if (game.isDownloaded) {
                        Icon(
                            imageVector = Icons.Default.CheckCircle,
                            contentDescription = "Скачано",
                            tint = EmeraldGreen,
                            modifier = Modifier.size(16.dp)
                        )
                    } else {
                        Icon(
                            imageVector = Icons.Default.Download,
                            contentDescription = "Скачать",
                            tint = NeonCyan,
                            modifier = Modifier.size(16.dp)
                        )
                    }
                }
            }
        }
    }
}
