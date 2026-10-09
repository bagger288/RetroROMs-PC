package com.example.model

enum class DownloadStatus {
    PENDING,
    DOWNLOADING,
    COMPLETED,
    FAILED,
    CANCELLED
}

data class ConsoleInfo(
    val slug: String,
    val name: String,
    val shortName: String,
    val category: String,
    val folderName: String,
    val section: String = "consoles",
    val isEnabled: Boolean = true,
    val order: Int = 0,
    val releaseYear: String = "",
    val iconKey: String = "gamepad",
    val romsCountEstimate: String = ""
)

data class RomFileVersion(
    val fid: String,
    val name: String,
    val size: String,
    val category: String = "Основные",
    val downloadUrl: String,
    val regionOrType: String = "USA"
)

data class GameCard(
    val id: String,
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
    val screenshotUrls: List<String> = emptyList(),
    val description: String = "",
    val downloadUrl: String = "",
    val mfileId: String? = null,
    val gamePageSlug: String? = null,
    val regions: List<String> = listOf("US"),
    val isFavorite: Boolean = false,
    val isDownloaded: Boolean = false
)

data class DownloadRecord(
    val id: Long,
    val gameId: String,
    val gameTitle: String,
    val consoleSlug: String,
    val consoleName: String,
    val fileName: String,
    val targetDirectory: String,
    val totalBytes: Long,
    val downloadedBytes: Long,
    val status: DownloadStatus,
    val downloadUrl: String,
    val timestamp: Long,
    val errorMessage: String? = null
)

data class CatalogCategory(
    val key: String,
    val label: String
)

data class GamesPageResult(
    val games: List<GameCard>,
    val currentPage: Int = 1,
    val totalPages: Int = 1,
    val hasNextPage: Boolean = false,
    val availableCategories: List<CatalogCategory> = emptyList()
)

data class ZipRomEntry(
    val entryName: String,
    val displayName: String,
    val sizeBytes: Long,
    val formattedSize: String
)

data class ZipExtractionRequest(
    val game: GameCard,
    val tempZipPath: String,
    val consoleFolderName: String,
    val customUriString: String?,
    val recordId: Long,
    val entries: List<ZipRomEntry>
)

sealed class DownloadExecutionResult {
    data class Completed(val path: String, val message: String) : DownloadExecutionResult()
    data class RequiresSelection(val request: ZipExtractionRequest) : DownloadExecutionResult()
    data class Failed(val error: Throwable) : DownloadExecutionResult()
}
