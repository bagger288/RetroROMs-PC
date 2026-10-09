package com.example.data.repository

import com.example.data.downloader.RomDownloader
import com.example.data.local.AppDatabase
import com.example.data.local.ConsoleEntity
import com.example.data.local.DownloadEntity
import com.example.data.local.GameEntity
import com.example.data.remote.EmuLandScraper
import com.example.model.CatalogCategory
import com.example.model.ConsoleInfo
import com.example.model.DownloadExecutionResult
import com.example.model.DownloadRecord
import com.example.model.DownloadStatus
import com.example.model.GameCard
import com.example.model.GamesPageResult
import com.example.model.RomFileVersion
import com.example.model.ZipExtractionRequest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

class EmuLandRepository(
    private val database: AppDatabase,
    private val scraper: EmuLandScraper,
    private val downloader: RomDownloader
) {

    private val consoleDao = database.consoleDao()
    private val gameDao = database.gameDao()
    private val downloadDao = database.downloadDao()

    init {
        CoroutineScope(Dispatchers.IO).launch {
            initDefaultDataIfNeeded()
        }
    }

    private suspend fun initDefaultDataIfNeeded() {
        val defaults = scraper.getDefaultConsoles()
        val validSlugs = defaults.map { it.slug }
        // Clean up any previously stored consoles without ROMs (e.g. nds, psp, dreamcast, ps2, etc.)
        consoleDao.retainOnlyConsoles(validSlugs)
        gameDao.deleteGamesForInvalidConsoles(validSlugs)

        val existingList = consoleDao.getAllConsolesList()
        if (existingList.isEmpty()) {
            consoleDao.insertConsoles(defaults.mapIndexed { index, item ->
                item.toEntity().copy(sortOrder = index)
            })
            for (c in defaults.take(4)) {
                val seedGames = scraper.getSeedGamesForConsole(c.slug, c.name, c.section)
                if (seedGames.isNotEmpty()) {
                    gameDao.insertGames(seedGames.map { it.toEntity() })
                }
            }
        } else {
            val existingSlugs = existingList.map { it.slug }.toSet()
            val missingDefaults = defaults.filter { it.slug !in existingSlugs }
            if (missingDefaults.isNotEmpty()) {
                val maxOrder = existingList.maxOfOrNull { it.sortOrder } ?: 0
                consoleDao.insertConsoles(missingDefaults.mapIndexed { idx, item ->
                    item.toEntity().copy(sortOrder = maxOrder + 1 + idx)
                })
            }
        }

        // Try syncing real consoles from web in background while preserving user preferences & custom order
        runCatching {
            val webConsoles = scraper.fetchConsoles()
            if (webConsoles.isNotEmpty()) {
                val existingMap = consoleDao.getAllConsolesList().associateBy { it.slug }
                val merged = webConsoles.map { c ->
                    val existing = existingMap[c.slug]
                    c.toEntity().copy(
                        isEnabled = existing?.isEnabled ?: c.isEnabled,
                        sortOrder = existing?.sortOrder ?: c.order
                    )
                }
                consoleDao.insertConsoles(merged)
            }
        }
    }

    fun getAllConsoles(): Flow<List<ConsoleInfo>> {
        return consoleDao.getAllConsoles().map { list ->
            list.map { it.toModel() }
        }
    }

    fun getEnabledConsoles(): Flow<List<ConsoleInfo>> {
        return consoleDao.getEnabledConsoles().map { list ->
            list.map { it.toModel() }
        }
    }

    suspend fun setConsoleEnabled(slug: String, isEnabled: Boolean) {
        consoleDao.setConsoleEnabled(slug, isEnabled)
    }

    suspend fun setAllConsolesEnabled(isEnabled: Boolean) {
        consoleDao.setAllConsolesEnabled(isEnabled)
    }

    suspend fun moveConsoleUp(slug: String) = withContext(Dispatchers.IO) {
        val list = consoleDao.getAllConsolesList()
        val index = list.indexOfFirst { it.slug == slug }
        if (index > 0) {
            val prev = list[index - 1]
            consoleDao.swapConsoleOrder(slug, prev.slug)
        }
    }

    suspend fun moveConsoleDown(slug: String) = withContext(Dispatchers.IO) {
        val list = consoleDao.getAllConsolesList()
        val index = list.indexOfFirst { it.slug == slug }
        if (index >= 0 && index < list.size - 1) {
            val next = list[index + 1]
            consoleDao.swapConsoleOrder(slug, next.slug)
        }
    }

    suspend fun moveConsoleToTop(slug: String) = withContext(Dispatchers.IO) {
        consoleDao.moveConsoleToTop(slug)
    }

    suspend fun resetConsolesOrder() = withContext(Dispatchers.IO) {
        val defaults = scraper.getDefaultConsoles()
        val defaultSlugs = defaults.map { it.slug }
        val current = consoleDao.getAllConsolesList()
        val currentMap = current.associateBy { it.slug }

        var order = 0
        defaultSlugs.forEach { slug ->
            if (currentMap.containsKey(slug)) {
                consoleDao.updateConsoleOrder(slug, order++)
            }
        }
        current.forEach { c ->
            if (c.slug !in defaultSlugs) {
                consoleDao.updateConsoleOrder(c.slug, order++)
            }
        }
    }

    suspend fun refreshConsolesFromWeb(): Result<List<ConsoleInfo>> = withContext(Dispatchers.IO) {
        try {
            val scraped = scraper.fetchConsoles()
            if (scraped.isNotEmpty()) {
                val existingMap = consoleDao.getAllConsolesList().associateBy { it.slug }
                val merged = scraped.map { c ->
                    val existing = existingMap[c.slug]
                    c.toEntity().copy(
                        isEnabled = existing?.isEnabled ?: c.isEnabled,
                        sortOrder = existing?.sortOrder ?: c.order
                    )
                }
                consoleDao.insertConsoles(merged)
            }
            Result.success(scraped)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun getInitialCategoriesForConsole(slug: String): List<CatalogCategory> {
        return scraper.getInitialCategoriesForConsole(slug)
    }

    fun getGamesForConsole(slug: String): Flow<List<GameCard>> {
        return gameDao.getGamesForConsole(slug).map { list ->
            list.map { it.toModel() }
        }
    }

    fun searchGames(query: String): Flow<List<GameCard>> {
        return gameDao.searchGames(query).map { list ->
            list.map { it.toModel() }
        }
    }

    fun getFavorites(): Flow<List<GameCard>> {
        return gameDao.getFavoriteGames().map { list ->
            list.map { it.toModel() }
        }
    }

    suspend fun toggleFavorite(gameId: String, current: Boolean) {
        gameDao.setFavorite(gameId, !current)
    }

    suspend fun refreshGamesForConsole(
        slug: String,
        consoleName: String,
        section: String = "consoles",
        category: String = "top",
        page: Int = 1
    ): Result<List<GameCard>> = withContext(Dispatchers.IO) {
        val result = fetchGamesPageForConsole(slug, consoleName, section, category, page)
        result.map { it.games }
    }

    suspend fun fetchGamesPageForConsole(
        slug: String,
        consoleName: String,
        section: String = "consoles",
        category: String = "top",
        page: Int = 1
    ): Result<GamesPageResult> = withContext(Dispatchers.IO) {
        try {
            val pageResult = scraper.fetchGamesPageForConsole(slug, consoleName, section, category, page)
            if (pageResult.games.isNotEmpty()) {
                val favIds = runCatching { gameDao.getFavoriteGameIds().toSet() }.getOrDefault(emptySet())
                val dlIds = runCatching { gameDao.getDownloadedGameIds().toSet() }.getOrDefault(emptySet())
                val updatedGames = pageResult.games.map { g ->
                    g.copy(
                        isFavorite = g.isFavorite || (g.id in favIds),
                        isDownloaded = g.isDownloaded || (g.id in dlIds)
                    )
                }
                gameDao.insertGames(updatedGames.map { it.toEntity() })
                Result.success(pageResult.copy(games = updatedGames))
            } else {
                Result.success(pageResult)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun searchGamesSiteWide(query: String): List<GameCard> = withContext(Dispatchers.IO) {
        val clean = query.trim()
        if (clean.isBlank()) return@withContext emptyList()
        try {
            val remoteResults = scraper.searchGamesSiteWide(clean)
            if (remoteResults.isNotEmpty()) {
                gameDao.insertGames(remoteResults.map { it.toEntity() })
                remoteResults
            } else {
                val localEntities = gameDao.searchGamesDirect(clean)
                localEntities.map { it.toModel() }
            }
        } catch (e: Exception) {
            val localEntities = gameDao.searchGamesDirect(clean)
            localEntities.map { it.toModel() }
        }
    }

    suspend fun loadFullGameDetails(game: GameCard): GameCard = withContext(Dispatchers.IO) {
        val enhanced = scraper.fetchGameCardDetails(game)
        gameDao.insertGame(enhanced.toEntity())
        enhanced
    }

    fun getCustomFolderUri(): String? = downloader.getCustomFolderUri()

    fun setCustomFolderUri(uriString: String?) {
        downloader.setCustomFolderUri(uriString)
    }

    suspend fun getRomVersions(game: GameCard): List<RomFileVersion> {
        val versions = scraper.fetchRomVersions(game)
        val mfileIdFromVersion = versions.firstOrNull { it.downloadUrl.contains("act=getmfl&id=") }
            ?.let { Regex("id=([0-9]+)").find(it.downloadUrl)?.groupValues?.get(1) }
        if (!mfileIdFromVersion.isNullOrBlank() && game.mfileId != mfileIdFromVersion) {
            try {
                gameDao.insertGame(game.copy(mfileId = mfileIdFromVersion).toEntity())
            } catch (_: Exception) {}
        }
        return versions
    }

    suspend fun downloadSpecificVersion(
        game: GameCard,
        version: RomFileVersion,
        consoleFolderName: String,
        onProgress: (percent: Int, downloaded: Long, total: Long) -> Unit
    ): DownloadExecutionResult {
        return downloader.downloadSpecificVersion(game, version, consoleFolderName, onProgress)
    }

    suspend fun downloadGame(
        game: GameCard,
        consoleFolderName: String,
        onProgress: (percent: Int, downloaded: Long, total: Long) -> Unit
    ): DownloadExecutionResult {
        return downloader.downloadGame(game, consoleFolderName, onProgress)
    }

    fun isAutoUnpackEnabled(): Boolean = downloader.isAutoUnpackEnabled()
    fun setAutoUnpackEnabled(enabled: Boolean) = downloader.setAutoUnpackEnabled(enabled)

    fun isDeleteZipAfterUnpack(): Boolean = downloader.isDeleteZipAfterUnpack()
    fun setDeleteZipAfterUnpack(enabled: Boolean) = downloader.setDeleteZipAfterUnpack(enabled)

    suspend fun extractSelectedZipEntries(
        request: ZipExtractionRequest,
        selectedEntryNames: List<String>
    ): Result<List<String>> {
        return downloader.extractSelectedEntries(request, selectedEntryNames)
    }

    suspend fun keepZipWithoutExtraction(
        request: ZipExtractionRequest
    ): Result<String> {
        return downloader.keepZipWithoutExtraction(request)
    }

    suspend fun dismissZipExtraction(request: ZipExtractionRequest) {
        downloader.dismissZipExtraction(request)
    }

    fun getAllDownloads(): Flow<List<DownloadRecord>> {
        return downloadDao.getAllDownloads().map { list ->
            list.map { it.toModel() }
        }
    }

    suspend fun deleteDownload(id: Long) {
        downloadDao.deleteDownload(id)
    }

    private fun ConsoleEntity.toModel(): ConsoleInfo = ConsoleInfo(
        slug = slug,
        name = name,
        shortName = shortName,
        category = category,
        folderName = folderName,
        section = section,
        isEnabled = isEnabled,
        order = sortOrder,
        releaseYear = releaseYear,
        romsCountEstimate = romsCountEstimate
    )

    private fun ConsoleInfo.toEntity(): ConsoleEntity = ConsoleEntity(
        slug = slug,
        name = name,
        shortName = shortName,
        category = category,
        folderName = folderName,
        section = section,
        isEnabled = isEnabled,
        sortOrder = order,
        releaseYear = releaseYear,
        romsCountEstimate = romsCountEstimate
    )

    private fun GameEntity.toModel(): GameCard = GameCard(
        id = id,
        consoleSlug = consoleSlug,
        consoleName = consoleName,
        section = section,
        title = title,
        originalTitle = originalTitle,
        genre = genre,
        year = year,
        publisher = publisher,
        developer = developer,
        rating = rating,
        fileSize = fileSize,
        coverUrl = coverUrl,
        screenshotUrls = if (screenshotUrlsRaw.isNotEmpty()) screenshotUrlsRaw.split("|||") else emptyList(),
        description = description,
        downloadUrl = downloadUrl,
        mfileId = mfileId,
        gamePageSlug = gamePageSlug,
        regions = if (regionsRaw.isNotEmpty()) regionsRaw.split(",") else listOf("US"),
        isFavorite = isFavorite,
        isDownloaded = isDownloaded
    )

    private fun GameCard.toEntity(): GameEntity = GameEntity(
        id = id,
        consoleSlug = consoleSlug,
        consoleName = consoleName,
        section = section,
        title = title,
        originalTitle = originalTitle,
        genre = genre,
        year = year,
        publisher = publisher,
        developer = developer,
        rating = rating,
        fileSize = fileSize,
        coverUrl = coverUrl,
        screenshotUrlsRaw = screenshotUrls.joinToString("|||"),
        description = description,
        downloadUrl = downloadUrl,
        mfileId = mfileId,
        gamePageSlug = gamePageSlug,
        regionsRaw = regions.joinToString(","),
        isFavorite = isFavorite,
        isDownloaded = isDownloaded
    )

    private fun DownloadEntity.toModel(): DownloadRecord = DownloadRecord(
        id = id,
        gameId = gameId,
        gameTitle = gameTitle,
        consoleSlug = consoleSlug,
        consoleName = consoleName,
        fileName = fileName,
        targetDirectory = targetDirectory,
        totalBytes = totalBytes,
        downloadedBytes = downloadedBytes,
        status = runCatching { DownloadStatus.valueOf(status) }.getOrDefault(DownloadStatus.PENDING),
        downloadUrl = downloadUrl,
        timestamp = timestamp,
        errorMessage = errorMessage
    )
}
