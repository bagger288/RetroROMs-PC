package com.example.data.local

import androidx.room.Entity
import androidx.room.PrimaryKey
import com.example.model.DownloadStatus

@Entity(tableName = "consoles")
data class ConsoleEntity(
    @PrimaryKey val slug: String,
    val name: String,
    val shortName: String,
    val category: String,
    val folderName: String,
    val section: String = "consoles",
    val isEnabled: Boolean = true,
    val sortOrder: Int = 0,
    val releaseYear: String = "",
    val romsCountEstimate: String = ""
)

@Entity(tableName = "games")
data class GameEntity(
    @PrimaryKey val id: String,
    val consoleSlug: String,
    val consoleName: String,
    val section: String = "consoles",
    val title: String,
    val originalTitle: String? = null,
    val genre: String = "Action",
    val year: String = "N/A",
    val publisher: String = "Unknown",
    val developer: String = "Unknown",
    val rating: Float = 4.8f,
    val fileSize: String = "ROM",
    val coverUrl: String? = null,
    val screenshotUrlsRaw: String = "",
    val description: String = "",
    val downloadUrl: String = "",
    val mfileId: String? = null,
    val gamePageSlug: String? = null,
    val regionsRaw: String = "US",
    val isFavorite: Boolean = false,
    val isDownloaded: Boolean = false,
    val lastUpdated: Long = System.currentTimeMillis()
)

@Entity(tableName = "downloads")
data class DownloadEntity(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val downloadManagerId: Long = -1L,
    val gameId: String,
    val gameTitle: String,
    val consoleSlug: String,
    val consoleName: String,
    val fileName: String,
    val targetDirectory: String,
    val totalBytes: Long = 0L,
    val downloadedBytes: Long = 0L,
    val status: String = DownloadStatus.PENDING.name,
    val downloadUrl: String = "",
    val timestamp: Long = System.currentTimeMillis(),
    val errorMessage: String? = null
)
