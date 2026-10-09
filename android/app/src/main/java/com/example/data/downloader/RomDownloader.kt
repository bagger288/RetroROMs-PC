package com.example.data.downloader

import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.util.Log
import androidx.documentfile.provider.DocumentFile
import com.example.data.local.DownloadDao
import com.example.data.local.DownloadEntity
import com.example.data.local.GameDao
import com.example.data.remote.EmuLandScraper
import com.example.model.DownloadExecutionResult
import com.example.model.DownloadRecord
import com.example.model.DownloadStatus
import com.example.model.GameCard
import com.example.model.RomFileVersion
import com.example.model.ZipExtractionRequest
import com.example.model.ZipRomEntry
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import java.io.File
import java.io.FileOutputStream
import java.io.OutputStream
import java.util.Locale
import java.util.concurrent.TimeUnit
import java.util.zip.ZipFile

class RomDownloader(
    private val context: Context,
    private val downloadDao: DownloadDao,
    private val gameDao: GameDao,
    private val scraper: EmuLandScraper
) {
    private val client: OkHttpClient = OkHttpClient.Builder()
        .connectTimeout(30, TimeUnit.SECONDS)
        .readTimeout(90, TimeUnit.SECONDS)
        .followRedirects(true)
        .followSslRedirects(true)
        .build()

    private val userAgent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36"

    private val prefs = context.getSharedPreferences("emuland_prefs", Context.MODE_PRIVATE)

    fun getCustomFolderUri(): String? {
        return prefs.getString("custom_folder_uri", null)
    }

    fun setCustomFolderUri(uriString: String?) {
        prefs.edit().putString("custom_folder_uri", uriString).apply()
    }

    fun isAutoUnpackEnabled(): Boolean {
        return prefs.getBoolean("auto_unpack_zip", true)
    }

    fun setAutoUnpackEnabled(enabled: Boolean) {
        prefs.edit().putBoolean("auto_unpack_zip", enabled).apply()
    }

    fun isDeleteZipAfterUnpack(): Boolean {
        return prefs.getBoolean("delete_zip_after_unpack", true)
    }

    fun setDeleteZipAfterUnpack(enabled: Boolean) {
        prefs.edit().putBoolean("delete_zip_after_unpack", enabled).apply()
    }

    private fun sanitizeFileName(name: String, extension: String = ".zip"): String {
        val clean = name.replace(Regex("[\\\\/:*?\"<>|]"), "_").trim()
        return if (clean.endsWith(extension, ignoreCase = true)) clean else "$clean$extension"
    }

    private fun formatFileSize(bytes: Long): String {
        if (bytes <= 0) return "0 B"
        val units = arrayOf("B", "KiB", "MiB", "GiB")
        val digitGroups = (Math.log10(bytes.toDouble()) / Math.log10(1024.0)).toInt().coerceIn(0, units.size - 1)
        val value = bytes / Math.pow(1024.0, digitGroups.toDouble())
        return String.format(Locale.US, "%.1f %s", value, units[digitGroups])
    }

    private fun getMimeType(fileName: String): String {
        val ext = fileName.substringAfterLast(".", "").lowercase()
        return when (ext) {
            "zip" -> "application/zip"
            "7z" -> "application/x-7z-compressed"
            "nes", "bin", "gen", "smd", "md", "sfc", "smc", "gba", "gb", "gbc", "z64", "n64", "nds", "sms", "gg", "pce" -> "application/octet-stream"
            else -> "application/octet-stream"
        }
    }

    /**
     * Inspects a ZIP archive to collect all ROM file entries, ignoring technical metadata.
     */
    fun scanZipRomEntries(zipFile: File): List<ZipRomEntry> {
        val entries = mutableListOf<ZipRomEntry>()
        try {
            ZipFile(zipFile).use { zip ->
                val nonRomExts = setOf(
                    "txt", "nfo", "url", "diz", "doc", "pdf", "html", "htm",
                    "png", "jpg", "jpeg", "bmp", "gif", "exe", "bat", "cmd"
                )
                val allList = zip.entries().toList()
                    .filter { !it.isDirectory && !it.name.contains("__MACOSX") && !it.name.endsWith(".DS_Store") }

                val romList = allList.filter {
                    val ext = it.name.substringAfterLast(".", "").lowercase()
                    ext !in nonRomExts
                }

                val targetList = if (romList.isNotEmpty()) romList else allList
                for (e in targetList) {
                    val simpleName = File(e.name).name
                    val rawSize = if (e.size >= 0) e.size else e.compressedSize
                    entries.add(
                        ZipRomEntry(
                            entryName = e.name,
                            displayName = simpleName,
                            sizeBytes = rawSize,
                            formattedSize = formatFileSize(rawSize)
                        )
                    )
                }
            }
        } catch (e: Exception) {
            Log.e("RomDownloader", "Failed to scan zip entries: ${e.message}", e)
        }
        return entries
    }

    /**
     * Downloads a game from Emu-Land and organizes it into the target folder.
     */
    suspend fun downloadGame(
        game: GameCard,
        consoleFolderName: String,
        onProgress: (percent: Int, downloaded: Long, total: Long) -> Unit = { _, _, _ -> }
    ): DownloadExecutionResult = withContext(Dispatchers.IO) {
        val fileName = sanitizeFileName(game.title)
        val customUriString = getCustomFolderUri()

        val recordId = downloadDao.insertDownload(
            DownloadEntity(
                id = 0,
                gameId = game.id,
                gameTitle = game.title,
                consoleSlug = game.consoleSlug,
                consoleName = game.consoleName,
                fileName = fileName,
                targetDirectory = customUriString ?: "/sdcard/Download/RetroROMs/$consoleFolderName",
                totalBytes = 0,
                downloadedBytes = 0,
                status = DownloadStatus.DOWNLOADING.name,
                downloadUrl = game.downloadUrl,
                timestamp = System.currentTimeMillis()
            )
        )

        try {
            val directDownloadUrl = scraper.resolveDirectDownloadUrl(game)
            val downloadUrl = directDownloadUrl ?: game.downloadUrl

            Log.d("RomDownloader", "Initiating download from: $downloadUrl")

            val request = Request.Builder()
                .url(downloadUrl)
                .header("User-Agent", userAgent)
                .header("Referer", "https://www.emu-land.net/${game.section}/${game.consoleSlug}/roms")
                .header("Accept", "*/*")
                .build()

            val response = client.newCall(request).execute()
            if (!response.isSuccessful || response.body == null) {
                downloadDao.updateProgress(recordId, DownloadStatus.FAILED.name, 0, 0)
                return@withContext DownloadExecutionResult.Failed(
                    Exception("Ошибка сервера ${response.code} при скачивании ${game.title}")
                )
            }

            val body = response.body!!
            val contentType = body.contentType()?.toString() ?: ""
            if (contentType.contains("text/html")) {
                downloadDao.updateProgress(recordId, DownloadStatus.FAILED.name, 0, 0)
                return@withContext DownloadExecutionResult.Failed(
                    Exception("Сервер вернул HTML страницу вместо файла игры.")
                )
            }

            val totalBytes = body.contentLength()
            var downloadedBytes = 0L

            val tempFile = File(context.cacheDir, "rom_dl_${System.currentTimeMillis()}_${sanitizeFileName(fileName)}")
            tempFile.outputStream().use { out ->
                body.byteStream().use { input ->
                    val buffer = ByteArray(8 * 1024)
                    var bytesRead: Int
                    var lastUpdate = System.currentTimeMillis()

                    while (input.read(buffer).also { bytesRead = it } != -1) {
                        out.write(buffer, 0, bytesRead)
                        downloadedBytes += bytesRead

                        val now = System.currentTimeMillis()
                        if (now - lastUpdate > 250) {
                            lastUpdate = now
                            val pct = if (totalBytes > 0) ((downloadedBytes * 100) / totalBytes).toInt() else 50
                            onProgress(pct, downloadedBytes, totalBytes)
                            downloadDao.updateProgress(recordId, DownloadStatus.DOWNLOADING.name, downloadedBytes, totalBytes)
                        }
                    }
                    out.flush()
                }
            }

            processDownloadedFile(
                game = game,
                tempFile = tempFile,
                fileName = fileName,
                consoleFolderName = consoleFolderName,
                customUriString = customUriString,
                recordId = recordId,
                downloadedBytes = downloadedBytes
            )

        } catch (e: Exception) {
            Log.e("RomDownloader", "Download failed for ${game.title}: ${e.message}", e)
            downloadDao.updateProgress(recordId, DownloadStatus.FAILED.name, 0, 0)
            DownloadExecutionResult.Failed(e)
        }
    }

    /**
     * Downloads a specific ROM version selected by the user.
     */
    suspend fun downloadSpecificVersion(
        game: GameCard,
        version: RomFileVersion,
        consoleFolderName: String,
        onProgress: (percent: Int, downloaded: Long, total: Long) -> Unit = { _, _, _ -> }
    ): DownloadExecutionResult = withContext(Dispatchers.IO) {
        val fileName = sanitizeFileName(version.name)
        val customUriString = getCustomFolderUri()

        val recordId = downloadDao.insertDownload(
            DownloadEntity(
                id = 0,
                gameId = game.id,
                gameTitle = "${game.title} [${version.name}]",
                consoleSlug = game.consoleSlug,
                consoleName = game.consoleName,
                fileName = fileName,
                targetDirectory = customUriString ?: "/sdcard/Download/RetroROMs/$consoleFolderName",
                totalBytes = 0,
                downloadedBytes = 0,
                status = DownloadStatus.DOWNLOADING.name,
                downloadUrl = version.downloadUrl,
                timestamp = System.currentTimeMillis()
            )
        )

        try {
            val referer = "https://www.emu-land.net/${game.section}/${game.consoleSlug}/roms"
            val directDownloadUrl = scraper.resolveDirectDownloadFromUrl(version.downloadUrl, referer)
                ?: version.downloadUrl

            Log.d("RomDownloader", "Downloading specific version from: $directDownloadUrl")

            val request = Request.Builder()
                .url(directDownloadUrl)
                .header("User-Agent", userAgent)
                .header("Referer", referer)
                .header("Accept", "*/*")
                .build()

            val response = client.newCall(request).execute()
            if (!response.isSuccessful || response.body == null) {
                downloadDao.updateProgress(recordId, DownloadStatus.FAILED.name, 0, 0)
                return@withContext DownloadExecutionResult.Failed(
                    Exception("Ошибка сервера ${response.code} при скачивании ${version.name}")
                )
            }

            val body = response.body!!
            val contentType = body.contentType()?.toString() ?: ""
            if (contentType.contains("text/html")) {
                downloadDao.updateProgress(recordId, DownloadStatus.FAILED.name, 0, 0)
                return@withContext DownloadExecutionResult.Failed(
                    Exception("Сервер вернул HTML вместо файла игры для ${version.name}.")
                )
            }

            val totalBytes = body.contentLength()
            var downloadedBytes = 0L

            val tempFile = File(context.cacheDir, "rom_ver_${System.currentTimeMillis()}_${sanitizeFileName(fileName)}")
            tempFile.outputStream().use { out ->
                body.byteStream().use { input ->
                    val buffer = ByteArray(8 * 1024)
                    var bytesRead: Int
                    var lastUpdate = System.currentTimeMillis()

                    while (input.read(buffer).also { bytesRead = it } != -1) {
                        out.write(buffer, 0, bytesRead)
                        downloadedBytes += bytesRead

                        val now = System.currentTimeMillis()
                        if (now - lastUpdate > 250) {
                            lastUpdate = now
                            val pct = if (totalBytes > 0) ((downloadedBytes * 100) / totalBytes).toInt() else 50
                            onProgress(pct, downloadedBytes, totalBytes)
                            downloadDao.updateProgress(recordId, DownloadStatus.DOWNLOADING.name, downloadedBytes, totalBytes)
                        }
                    }
                    out.flush()
                }
            }

            processDownloadedFile(
                game = game,
                tempFile = tempFile,
                fileName = fileName,
                consoleFolderName = consoleFolderName,
                customUriString = customUriString,
                recordId = recordId,
                downloadedBytes = downloadedBytes
            )

        } catch (e: Exception) {
            Log.e("RomDownloader", "Failed to download specific version ${version.name}: ${e.message}", e)
            downloadDao.updateProgress(recordId, DownloadStatus.FAILED.name, 0, 0)
            DownloadExecutionResult.Failed(e)
        }
    }

    /**
     * Inspects downloaded payload. If it is a ZIP archive and auto-unpacking is enabled:
     * - If exactly 1 ROM: extracts it directly and deletes the ZIP.
     * - If > 1 ROMs: requests user selection of which ROMs to extract.
     * - Otherwise: writes the ZIP file as-is.
     */
    private suspend fun processDownloadedFile(
        game: GameCard,
        tempFile: File,
        fileName: String,
        consoleFolderName: String,
        customUriString: String?,
        recordId: Long,
        downloadedBytes: Long
    ): DownloadExecutionResult = withContext(Dispatchers.IO) {
        val autoUnpack = isAutoUnpackEnabled()
        val isZip = fileName.endsWith(".zip", ignoreCase = true)

        if (autoUnpack && isZip) {
            val entries = scanZipRomEntries(tempFile)
            if (entries.size == 1) {
                // Exactly 1 ROM entry: extract directly!
                val singleEntry = entries.first()
                val simpleName = singleEntry.displayName
                val outStreamAndPath = createTargetOutputStream(
                    customUriString = customUriString,
                    consoleFolderName = consoleFolderName,
                    fileName = simpleName,
                    mimeType = getMimeType(simpleName)
                )

                try {
                    ZipFile(tempFile).use { zip ->
                        val entry = zip.getEntry(singleEntry.entryName)
                        if (entry != null) {
                            outStreamAndPath.first.use { out ->
                                zip.getInputStream(entry).use { input ->
                                    input.copyTo(out)
                                }
                            }
                        }
                    }

                    if (isDeleteZipAfterUnpack()) {
                        tempFile.delete()
                    }

                    downloadDao.updateProgress(recordId, DownloadStatus.COMPLETED.name, downloadedBytes, downloadedBytes)
                    gameDao.setDownloaded(game.id, true)
                    Log.d("RomDownloader", "Extracted single ROM: $simpleName to ${outStreamAndPath.second}")
                    return@withContext DownloadExecutionResult.Completed(
                        path = outStreamAndPath.second,
                        message = "Распаковано: $simpleName"
                    )
                } catch (e: Exception) {
                    Log.w("RomDownloader", "Extraction failed, falling back to writing whole zip: ${e.message}")
                }
            } else if (entries.size > 1) {
                // Multiple ROMs found! Prompt user selection.
                downloadDao.updateProgress(recordId, DownloadStatus.DOWNLOADING.name, downloadedBytes, downloadedBytes)
                Log.d("RomDownloader", "Multiple ROMs (${entries.size}) found in $fileName for ${game.title}")
                return@withContext DownloadExecutionResult.RequiresSelection(
                    ZipExtractionRequest(
                        game = game,
                        tempZipPath = tempFile.absolutePath,
                        consoleFolderName = consoleFolderName,
                        customUriString = customUriString,
                        recordId = recordId,
                        entries = entries
                    )
                )
            }
        }

        // Save original file directly
        val outStreamAndPath = createTargetOutputStream(
            customUriString = customUriString,
            consoleFolderName = consoleFolderName,
            fileName = fileName,
            mimeType = getMimeType(fileName)
        )
        outStreamAndPath.first.use { out ->
            tempFile.inputStream().use { input ->
                input.copyTo(out)
            }
        }
        tempFile.delete()

        downloadDao.updateProgress(recordId, DownloadStatus.COMPLETED.name, downloadedBytes, downloadedBytes)
        gameDao.setDownloaded(game.id, true)
        Log.d("RomDownloader", "Successfully saved: ${outStreamAndPath.second}")
        DownloadExecutionResult.Completed(
            path = outStreamAndPath.second,
            message = "Скачано: $fileName"
        )
    }

    /**
     * Extracts selected ROM entries from a multi-file ZIP archive into the console folder.
     */
    suspend fun extractSelectedEntries(
        request: ZipExtractionRequest,
        selectedEntryNames: List<String>
    ): Result<List<String>> = withContext(Dispatchers.IO) {
        val tempFile = File(request.tempZipPath)
        if (!tempFile.exists()) {
            return@withContext Result.failure(Exception("Временный файл архива не найден"))
        }

        val extracted = mutableListOf<String>()
        try {
            ZipFile(tempFile).use { zip ->
                for (entryName in selectedEntryNames) {
                    val zipEntry = zip.getEntry(entryName) ?: continue
                    val simpleName = File(zipEntry.name).name
                    val outStreamAndPath = createTargetOutputStream(
                        customUriString = request.customUriString,
                        consoleFolderName = request.consoleFolderName,
                        fileName = simpleName,
                        mimeType = getMimeType(simpleName)
                    )
                    outStreamAndPath.first.use { out ->
                        zip.getInputStream(zipEntry).use { input ->
                            input.copyTo(out)
                        }
                    }
                    extracted.add(outStreamAndPath.second)
                }
            }

            if (isDeleteZipAfterUnpack()) {
                tempFile.delete()
            }

            val totalSize = tempFile.length()
            downloadDao.updateProgress(request.recordId, DownloadStatus.COMPLETED.name, totalSize, totalSize)
            gameDao.setDownloaded(request.game.id, true)

            Result.success(extracted)
        } catch (e: Exception) {
            Log.e("RomDownloader", "Error during zip extraction: ${e.message}", e)
            Result.failure(e)
        }
    }

    /**
     * Keeps the original ZIP archive intact in the destination folder without unpacking.
     */
    suspend fun keepZipWithoutExtraction(
        request: ZipExtractionRequest
    ): Result<String> = withContext(Dispatchers.IO) {
        val tempFile = File(request.tempZipPath)
        if (!tempFile.exists()) {
            return@withContext Result.failure(Exception("Временный файл архива не найден"))
        }

        try {
            val fileName = sanitizeFileName(request.game.title)
            val outStreamAndPath = createTargetOutputStream(
                customUriString = request.customUriString,
                consoleFolderName = request.consoleFolderName,
                fileName = fileName,
                mimeType = "application/zip"
            )
            outStreamAndPath.first.use { out ->
                tempFile.inputStream().use { input ->
                    input.copyTo(out)
                }
            }
            tempFile.delete()

            val totalSize = tempFile.length()
            downloadDao.updateProgress(request.recordId, DownloadStatus.COMPLETED.name, totalSize, totalSize)
            gameDao.setDownloaded(request.game.id, true)
            Result.success(outStreamAndPath.second)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    /**
     * Cancels pending ZIP extraction and cleans up the temporary downloaded file.
     */
    suspend fun dismissZipExtraction(request: ZipExtractionRequest) = withContext(Dispatchers.IO) {
        try {
            File(request.tempZipPath).delete()
        } catch (_: Exception) {}
        downloadDao.updateProgress(request.recordId, DownloadStatus.CANCELLED.name, 0, 0)
    }

    /**
     * Creates output stream writing to either custom SAF folder or public /sdcard/Download/RetroROMs/<consoleFolderName>/
     */
    private fun createTargetOutputStream(
        customUriString: String?,
        consoleFolderName: String,
        fileName: String,
        mimeType: String = "application/octet-stream"
    ): Pair<OutputStream, String> {
        // Option 1: Custom SAF folder chosen by the user
        if (!customUriString.isNullOrBlank()) {
            try {
                val rootDoc = DocumentFile.fromTreeUri(context, Uri.parse(customUriString))
                if (rootDoc != null && rootDoc.canWrite()) {
                    var consoleDoc = rootDoc.findFile(consoleFolderName)
                    if (consoleDoc == null || !consoleDoc.isDirectory) {
                        consoleDoc = rootDoc.createDirectory(consoleFolderName)
                    }
                    val targetDoc = consoleDoc?.createFile(mimeType, fileName)
                    if (targetDoc != null) {
                        val stream = context.contentResolver.openOutputStream(targetDoc.uri)
                        if (stream != null) {
                            val pathName = "${rootDoc.name ?: "Custom"}/$consoleFolderName/$fileName"
                            return stream to pathName
                        }
                    }
                }
            } catch (e: Exception) {
                Log.w("RomDownloader", "SAF custom folder failed: ${e.message}, falling back to public Download")
            }
        }

        // Option 2: Android 10+ MediaStore.Downloads
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            try {
                val contentValues = ContentValues().apply {
                    put(MediaStore.MediaColumns.DISPLAY_NAME, fileName)
                    put(MediaStore.MediaColumns.MIME_TYPE, mimeType)
                    put(MediaStore.MediaColumns.RELATIVE_PATH, "${Environment.DIRECTORY_DOWNLOADS}/RetroROMs/$consoleFolderName")
                }
                val uri = context.contentResolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, contentValues)
                if (uri != null) {
                    val stream = context.contentResolver.openOutputStream(uri)
                    if (stream != null) {
                        return stream to "/sdcard/Download/RetroROMs/$consoleFolderName/$fileName"
                    }
                }
            } catch (e: Exception) {
                Log.w("RomDownloader", "MediaStore.Downloads failed: ${e.message}")
            }
        }

        // Option 3: Direct File in /sdcard/Download/RetroROMs/<consoleFolderName>/
        val publicDownloads = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
        val retroDir = File(publicDownloads, "RetroROMs/$consoleFolderName")
        if (!retroDir.exists()) retroDir.mkdirs()
        val file = File(retroDir, fileName)
        return try {
            FileOutputStream(file) to file.absolutePath
        } catch (_: Exception) {
            val fallbackDir = File(context.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS), "RetroROMs/$consoleFolderName")
            if (!fallbackDir.exists()) fallbackDir.mkdirs()
            val fallbackFile = File(fallbackDir, fileName)
            FileOutputStream(fallbackFile) to fallbackFile.absolutePath
        }
    }
}
