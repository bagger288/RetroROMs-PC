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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Archive
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Download
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.example.model.ZipExtractionRequest
import com.example.ui.theme.ArcadeCardBorder
import com.example.ui.theme.ArcadeSurface
import com.example.ui.theme.ArcadeSurfaceContainer
import com.example.ui.theme.ArcadeSurfaceVariant
import com.example.ui.theme.GoldenAmber
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.OnPrimary
import com.example.ui.theme.TextMuted
import com.example.ui.theme.TextPrimary
import com.example.ui.theme.TextSecondary

@Composable
fun ZipExtractionDialog(
    request: ZipExtractionRequest,
    onExtractSelected: (selectedEntryNames: List<String>) -> Unit,
    onKeepZip: () -> Unit,
    onDismiss: () -> Unit
) {
    // No smart pre-selection: initially all checkboxes can be toggled by the user
    var selectedEntries by remember { mutableStateOf(emptySet<String>()) }

    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(
            usePlatformDefaultWidth = false,
            dismissOnBackPress = true,
            dismissOnClickOutside = true
        )
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth(0.94f)
                .clip(RoundedCornerShape(20.dp))
                .background(ArcadeSurface)
                .border(1.dp, NeonCyan.copy(alpha = 0.4f), RoundedCornerShape(20.dp))
                .padding(20.dp)
        ) {
            Column(modifier = Modifier.fillMaxWidth()) {
                // Header: Title & Close Button
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.weight(1f)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(36.dp)
                                .clip(RoundedCornerShape(10.dp))
                                .background(NeonCyan.copy(alpha = 0.15f)),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                imageVector = Icons.Default.Archive,
                                contentDescription = null,
                                tint = NeonCyan,
                                modifier = Modifier.size(20.dp)
                            )
                        }
                        Spacer(modifier = Modifier.width(10.dp))
                        Column {
                            Text(
                                text = "Файлы в архиве",
                                color = TextPrimary,
                                fontSize = 17.sp,
                                fontWeight = FontWeight.Bold
                            )
                            Text(
                                text = "${request.game.title} • ${request.entries.size} РОМов",
                                color = GoldenAmber,
                                fontSize = 12.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                    }

                    IconButton(
                        onClick = onDismiss,
                        modifier = Modifier.size(32.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Close,
                            contentDescription = "Закрыть",
                            tint = TextMuted,
                            modifier = Modifier.size(20.dp)
                        )
                    }
                }

                Spacer(modifier = Modifier.height(12.dp))

                Text(
                    text = "В архиве найдено несколько версий игры. Отметьте те, которые хотите распаковать в папку консоли:",
                    color = TextSecondary,
                    fontSize = 12.sp,
                    lineHeight = 16.sp
                )

                Spacer(modifier = Modifier.height(10.dp))

                // Selection Helper Toolbar
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "Выбрано: ${selectedEntries.size} из ${request.entries.size}",
                        color = if (selectedEntries.isNotEmpty()) NeonCyan else TextMuted,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold
                    )

                    Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                        TextButton(
                            onClick = {
                                selectedEntries = request.entries.map { it.entryName }.toSet()
                            }
                        ) {
                            Text("Выбрать все", fontSize = 11.sp, color = NeonCyan)
                        }

                        if (selectedEntries.isNotEmpty()) {
                            TextButton(
                                onClick = {
                                    selectedEntries = emptySet()
                                }
                            ) {
                                Text("Снять все", fontSize = 11.sp, color = TextMuted)
                            }
                        }
                    }
                }

                Spacer(modifier = Modifier.height(6.dp))

                // Scrollable List of ROM Entries
                LazyColumn(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(max = 280.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(request.entries, key = { it.entryName }) { entry ->
                        val isSelected = entry.entryName in selectedEntries

                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(12.dp))
                                .background(if (isSelected) ArcadeSurfaceContainer else ArcadeSurfaceVariant)
                                .border(
                                    1.dp,
                                    if (isSelected) NeonCyan.copy(alpha = 0.5f) else ArcadeCardBorder,
                                    RoundedCornerShape(12.dp)
                                )
                                .clickable {
                                    selectedEntries = if (isSelected) {
                                        selectedEntries - entry.entryName
                                    } else {
                                        selectedEntries + entry.entryName
                                    }
                                }
                                .padding(horizontal = 12.dp, vertical = 10.dp)
                        ) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Checkbox(
                                    checked = isSelected,
                                    onCheckedChange = { checked ->
                                        selectedEntries = if (checked) {
                                            selectedEntries + entry.entryName
                                        } else {
                                            selectedEntries - entry.entryName
                                        }
                                    },
                                    colors = CheckboxDefaults.colors(
                                        checkedColor = NeonCyan,
                                        checkmarkColor = OnPrimary,
                                        uncheckedColor = TextMuted
                                    ),
                                    modifier = Modifier.size(20.dp)
                                )

                                Spacer(modifier = Modifier.width(10.dp))

                                Column(modifier = Modifier.weight(1f)) {
                                    Text(
                                        text = entry.displayName,
                                        color = if (isSelected) TextPrimary else TextSecondary,
                                        fontSize = 12.5.sp,
                                        fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                                        maxLines = 2,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                    Spacer(modifier = Modifier.height(2.dp))
                                    Text(
                                        text = entry.formattedSize,
                                        color = TextMuted,
                                        fontSize = 11.sp
                                    )
                                }
                            }
                        }
                    }
                }

                Spacer(modifier = Modifier.height(16.dp))

                // Action Buttons
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    // Extract Selected Button
                    Button(
                        onClick = {
                            if (selectedEntries.isNotEmpty()) {
                                onExtractSelected(selectedEntries.toList())
                            }
                        },
                        enabled = selectedEntries.isNotEmpty(),
                        modifier = Modifier
                            .weight(1f)
                            .height(42.dp)
                            .testTag("extract_selected_roms_btn"),
                        shape = RoundedCornerShape(10.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = NeonCyan,
                            contentColor = OnPrimary,
                            disabledContainerColor = ArcadeSurfaceContainer,
                            disabledContentColor = TextMuted
                        )
                    ) {
                        Icon(
                            imageVector = Icons.Default.Check,
                            contentDescription = null,
                            modifier = Modifier.size(16.dp)
                        )
                        Spacer(modifier = Modifier.width(6.dp))
                        Text(
                            text = if (selectedEntries.isNotEmpty()) "Извлечь (${selectedEntries.size})" else "Выберите файлы",
                            fontSize = 12.sp,
                            fontWeight = FontWeight.Bold
                        )
                    }

                    // Keep original ZIP button
                    OutlinedButton(
                        onClick = onKeepZip,
                        modifier = Modifier
                            .height(42.dp)
                            .testTag("keep_zip_file_btn"),
                        shape = RoundedCornerShape(10.dp),
                        colors = ButtonDefaults.outlinedButtonColors(contentColor = TextPrimary),
                        border = androidx.compose.foundation.BorderStroke(1.dp, ArcadeCardBorder)
                    ) {
                        Text("Оставить ZIP", fontSize = 11.5.sp)
                    }
                }
            }
        }
    }
}
