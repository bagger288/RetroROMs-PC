package com.example.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
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
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Gamepad
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import com.example.model.ConsoleInfo
import com.example.ui.components.ConsolePlatformBadge
import com.example.ui.theme.ArcadeCardBorder
import com.example.ui.theme.ArcadeSurface
import com.example.ui.theme.ArcadeSurfaceContainer
import com.example.ui.theme.ArcadeSurfaceVariant
import com.example.ui.theme.NeonCyan
import com.example.ui.theme.TextMuted
import com.example.ui.theme.TextPrimary
import com.example.ui.theme.TextSecondary

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ManageConsolesDialog(
    consoles: List<ConsoleInfo>,
    onToggleConsole: (slug: String, isEnabled: Boolean) -> Unit,
    onSetAllConsoles: (isEnabled: Boolean) -> Unit,
    onDismiss: () -> Unit
) {
    Dialog(onDismissRequest = onDismiss) {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
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
                    Column {
                        Text(
                            text = "Manage Platforms",
                            color = TextPrimary,
                            fontSize = 18.sp,
                            fontWeight = FontWeight.Bold
                        )
                        Text(
                            text = "Select which consoles appear on Home",
                            color = TextMuted,
                            fontSize = 12.sp
                        )
                    }

                    IconButton(
                        onClick = onDismiss,
                        modifier = Modifier
                            .size(32.dp)
                            .clip(RoundedCornerShape(8.dp))
                            .background(ArcadeSurfaceVariant)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Close,
                            contentDescription = "Close",
                            tint = TextSecondary,
                            modifier = Modifier.size(16.dp)
                        )
                    }
                }

                Spacer(modifier = Modifier.height(14.dp))

                // Quick Presets Row
                Text(
                    text = "QUICK PRESETS",
                    color = TextMuted,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 0.5.sp
                )
                Spacer(modifier = Modifier.height(6.dp))

                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    PresetChip(text = "All Consoles") {
                        onSetAllConsoles(true)
                    }
                    PresetChip(text = "16-Bit Systems") {
                        consoles.forEach {
                            val shouldEnable = it.category.contains("16-bit", ignoreCase = true)
                            onToggleConsole(it.slug, shouldEnable)
                        }
                    }
                    PresetChip(text = "Handhelds Only") {
                        consoles.forEach {
                            val shouldEnable = it.category.contains("Handheld", ignoreCase = true)
                            onToggleConsole(it.slug, shouldEnable)
                        }
                    }
                    PresetChip(text = "Reset Default") {
                        val defaultSlugs = setOf("dendy", "genesis", "snes", "gba", "psx", "n64", "gb", "gbc")
                        consoles.forEach {
                            onToggleConsole(it.slug, it.slug in defaultSlugs)
                        }
                    }
                }

                Spacer(modifier = Modifier.height(16.dp))

                // Consoles List
                LazyColumn(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(max = 380.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    items(consoles, key = { it.slug }) { console ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(12.dp))
                                .background(ArcadeSurfaceVariant)
                                .border(
                                    1.dp,
                                    if (console.isEnabled) NeonCyan.copy(alpha = 0.35f) else ArcadeCardBorder,
                                    RoundedCornerShape(12.dp)
                                )
                                .clickable { onToggleConsole(console.slug, !console.isEnabled) }
                                .padding(horizontal = 14.dp, vertical = 10.dp),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Column(modifier = Modifier.weight(1f)) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    ConsolePlatformBadge(
                                        slug = console.slug,
                                        name = console.shortName
                                    )
                                    Spacer(modifier = Modifier.width(8.dp))
                                    Text(
                                        text = console.category,
                                        color = TextMuted,
                                        fontSize = 11.sp
                                    )
                                }
                                Spacer(modifier = Modifier.height(4.dp))
                                Text(
                                    text = "Folder: Downloads/RetroROMs/${console.folderName}/",
                                    color = TextSecondary,
                                    fontSize = 11.sp
                                )
                            }

                            Switch(
                                checked = console.isEnabled,
                                onCheckedChange = { onToggleConsole(console.slug, it) },
                                colors = SwitchDefaults.colors(
                                    checkedThumbColor = com.example.ui.theme.LocalAppColors.current.onPrimaryAccent,
                                    checkedTrackColor = NeonCyan,
                                    uncheckedThumbColor = TextMuted,
                                    uncheckedTrackColor = ArcadeSurfaceContainer
                                )
                            )
                        }
                    }
                }

                Spacer(modifier = Modifier.height(16.dp))

                Button(
                    onClick = onDismiss,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(48.dp)
                        .testTag("apply_consoles_btn"),
                    shape = RoundedCornerShape(12.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = NeonCyan,
                        contentColor = com.example.ui.theme.LocalAppColors.current.onPrimaryAccent
                    )
                ) {
                    Text(
                        text = "APPLY & CLOSE",
                        fontWeight = FontWeight.Bold,
                        fontSize = 13.sp
                    )
                }
            }
        }
    }
}

@Composable
private fun PresetChip(
    text: String,
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(8.dp))
            .background(ArcadeSurfaceContainer)
            .border(1.dp, ArcadeCardBorder, RoundedCornerShape(8.dp))
            .clickable { onClick() }
            .padding(horizontal = 10.dp, vertical = 6.dp)
    ) {
        Text(
            text = text,
            color = NeonCyan,
            fontSize = 11.sp,
            fontWeight = FontWeight.SemiBold
        )
    }
}
