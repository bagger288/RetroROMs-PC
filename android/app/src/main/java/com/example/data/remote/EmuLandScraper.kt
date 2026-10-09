package com.example.data.remote

import android.util.Log
import com.example.model.CatalogCategory
import com.example.model.ConsoleInfo
import com.example.model.GameCard
import com.example.model.GamesPageResult
import com.example.model.RomFileVersion
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONObject
import org.jsoup.Jsoup
import java.util.concurrent.TimeUnit

class EmuLandScraper {

    private val client: OkHttpClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(25, TimeUnit.SECONDS)
        .followRedirects(true)
        .followSslRedirects(true)
        .build()

    // Dedicated non-redirecting client to intercept HTTP 302 Location header
    private val noRedirectClient: OkHttpClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(25, TimeUnit.SECONDS)
        .followRedirects(false)
        .followSslRedirects(false)
        .build()

    private val baseUrl = "https://www.emu-land.net"
    private val userAgent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36"

    /**
     * Normalizes image and media URLs so Coil and Android can load them without crashing.
     */
    fun normalizeImageUrl(rawUrl: String?): String? {
        if (rawUrl.isNullOrBlank()) return null
        var url = rawUrl.trim()
            .replace("&amp;", "&")
            .replace("\\/", "/")

        if (url.startsWith("//")) {
            url = "https:$url"
        } else if (url.startsWith("/")) {
            url = "$baseUrl$url"
        }

        // Encode unescaped spaces
        return url.replace(" ", "%20")
    }

    /**
     * Scrapes or builds the complete console list from emu-land.net/consoles and /portable
     */
    suspend fun fetchConsoles(): List<ConsoleInfo> = withContext(Dispatchers.IO) {
        val consoles = mutableListOf<ConsoleInfo>()
        val seenSlugs = mutableSetOf<String>()

        try {
            // 1. Fetch home consoles
            scrapeConsoleSection("/consoles", "consoles", consoles, seenSlugs)
            // 2. Fetch portable consoles
            scrapeConsoleSection("/portable", "portable", consoles, seenSlugs)
        } catch (e: Exception) {
            Log.w("EmuLandScraper", "Network error fetching consoles list: ${e.message}")
        }

        if (consoles.isEmpty()) {
            consoles.addAll(getDefaultConsoles())
        } else {
            // Ensure essential consoles are present
            for (def in getDefaultConsoles()) {
                if (def.slug !in seenSlugs) {
                    consoles.add(def.copy(order = consoles.size))
                    seenSlugs.add(def.slug)
                }
            }
        }

        consoles.sortedBy { it.order }
    }

    companion object {
        val CONSOLES_WITH_GAMES = setOf(
            "dendy", "genesis", "snes", "n64", "psx", "32x", "segacd", "3do",
            "pce", "pcecd", "sms", "sg-1000", "famicom_disk_system", "neogeocd",
            "2600", "5200", "7800", "jaguar", "coleco", "vectrex", "intellivision",
            "arcadia", "chaf", "gba", "gb", "gbc", "gg", "lynx", "ngp", "ws",
            "vboy", "pmini", "sv"
        )
    }

    private fun scrapeConsoleSection(
        path: String,
        section: String,
        outList: MutableList<ConsoleInfo>,
        seenSlugs: MutableSet<String>
    ) {
        try {
            val request = Request.Builder()
                .url("$baseUrl$path")
                .header("User-Agent", userAgent)
                .build()

            val response = client.newCall(request).execute()
            if (response.isSuccessful) {
                val html = response.body?.string() ?: ""
                val doc = Jsoup.parse(html, baseUrl)
                val links = doc.select("a[href*=$path/]")

                for (link in links) {
                    val href = link.attr("href")
                    val match = Regex("$path/([a-zA-Z0-9_-]+)/?.*").find(href)
                    if (match != null) {
                        val slug = match.groupValues[1].lowercase()
                        if (slug !in seenSlugs && slug in CONSOLES_WITH_GAMES) {
                            val title = link.text().trim()
                            if (title.isNotEmpty()) {
                                seenSlugs.add(slug)
                                outList.add(mapSlugToConsoleInfo(slug, title, section, outList.size))
                            }
                        }
                    }
                }
            }
        } catch (e: Exception) {
            Log.w("EmuLandScraper", "Error in scrapeConsoleSection $path: ${e.message}")
        }
    }

    fun getGameSubpath(slug: String): String {
        return when (slug.lowercase()) {
            "psx" -> "iso"
            "3do", "segacd", "pcecd", "neogeocd", "famicom_disk_system", "sg-1000", "gb", "gbc" -> "games"
            else -> "roms"
        }
    }

    fun supportsTopCategory(slug: String): Boolean {
        return slug.lowercase() in setOf(
            "dendy", "genesis", "snes", "n64", "segacd", "pce", "pcecd", "3do", "sms",
            "famicom_disk_system", "ws", "gb", "gbc", "gba", "gg", "ngp"
        )
    }

    fun getInitialCategoriesForConsole(slug: String): List<CatalogCategory> {
        val s = slug.lowercase()
        return when (s) {
            "dendy", "genesis", "snes", "gb", "gbc" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("best", "★ Лучшие"),
                CatalogCategory("homebrew", "🛠 Homebrew"),
                CatalogCategory("misc", "📁 Прочее"),
                CatalogCategory("0-9", "0-9")
            ) + ('a'..'z').map { CatalogCategory(it.toString(), it.uppercase()) }

            "gba" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("best", "★ Лучшие"),
                CatalogCategory("misc", "📁 Прочее"),
                CatalogCategory("0-9", "0-9")
            ) + ('a'..'z').map { CatalogCategory(it.toString(), it.uppercase()) }

            "n64" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("rating", "★ Рейтинг"),
                CatalogCategory("misc", "📁 Прочее"),
                CatalogCategory("0-9", "0-9")
            ) + ('a'..'z').map { CatalogCategory(it.toString(), it.uppercase()) }

            "segacd" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("best", "★ Лучшие"),
                CatalogCategory("misc", "📁 Прочее"),
                CatalogCategory("0-9", "0-9")
            ) + ('a'..'z').filter { it !in listOf('o', 'x', 'z') }.map { CatalogCategory(it.toString(), it.uppercase()) }

            "3do" -> listOf(
                CatalogCategory("top", "🔥 Популярные")
            ) + ('a'..'z').filter { it !in listOf('u', 'x') }.map { CatalogCategory(it.toString(), it.uppercase()) }

            "pce" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("best", "★ Лучшие"),
                CatalogCategory("misc", "📁 Прочее"),
                CatalogCategory("0-9", "0-9")
            ) + ('a'..'z').filter { it != 'u' }.map { CatalogCategory(it.toString(), it.uppercase()) }

            "pcecd" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("best", "★ Лучшие"),
                CatalogCategory("0-9", "0-9")
            ) + ('a'..'z').filter { it != 'o' }.map { CatalogCategory(it.toString(), it.uppercase()) }

            "sms" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("best", "★ Лучшие"),
                CatalogCategory("misc", "📁 Прочее"),
                CatalogCategory("0-9", "0-9")
            ) + ('a'..'z').map { CatalogCategory(it.toString(), it.uppercase()) }

            "famicom_disk_system" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("rating", "★ Рейтинг"),
                CatalogCategory("misc", "📁 Прочее")
            ) + ('a'..'z').filter { it != 'q' }.map { CatalogCategory(it.toString(), it.uppercase()) }

            "neogeocd" -> listOf(
                CatalogCategory("best", "★ Лучшие"),
                CatalogCategory("0-9", "0-9")
            ) + listOf('a', 'b', 'c', 'd', 'f', 'g', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'z').map { CatalogCategory(it.toString(), it.uppercase()) }

            "jaguar" -> listOf(
                CatalogCategory("misc", "📁 Прочее"),
                CatalogCategory("0-9", "0-9")
            ) + ('a'..'z').filter { it !in listOf('x', 'y') }.map { CatalogCategory(it.toString(), it.uppercase()) }

            "gg" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("rating", "★ Рейтинг"),
                CatalogCategory("0-9", "0-9")
            ) + ('a'..'z').map { CatalogCategory(it.toString(), it.uppercase()) }

            "ngp" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("rating", "★ Рейтинг")
            ) + listOf('b', 'c', 'd', 'e', 'f', 'g', 'i', 'k', 'l', 'm', 'n', 'o', 'p', 'r', 's').map { CatalogCategory(it.toString(), it.uppercase()) }

            "ws" -> listOf(
                CatalogCategory("top", "🔥 Популярные"),
                CatalogCategory("rating", "★ Рейтинг"),
                CatalogCategory("misc", "📁 Прочее")
            ) + ('a'..'z').filter { it !in listOf('j', 'q', 'z') }.map { CatalogCategory(it.toString(), it.uppercase()) }

            // Consoles with all games on a single page without pagelist_top
            else -> listOf(CatalogCategory("all", "Все игры"))
        }
    }

    fun extractCategoriesFromDoc(doc: org.jsoup.nodes.Document, consoleSlug: String, subpath: String): List<CatalogCategory> {
        val pagelist = doc.selectFirst("#pagelist_top") ?: doc.selectFirst(".pagelist")
        if (pagelist == null) {
            return getInitialCategoriesForConsole(consoleSlug)
        }

        val categories = mutableListOf<CatalogCategory>()
        val seenKeys = mutableSetOf<String>()

        // 1. .add links (special filters: top, best, rating, homebrew, misc)
        val addLinks = pagelist.select(".add a")
        val addItems = mutableListOf<Pair<String, String>>()
        for (a in addLinks) {
            val href = a.attr("href").trimEnd('/')
            val key = href.substringAfterLast('/').lowercase()
            val text = a.text().trim()
            if (key.isNotBlank()) {
                addItems.add(key to text)
            }
        }
        val orderMap = mapOf("top" to 1, "best" to 2, "rating" to 3, "homebrew" to 4, "misc" to 5)
        addItems.sortBy { orderMap[it.first] ?: 99 }
        for ((k, t) in addItems) {
            val label = when (k) {
                "top" -> "🔥 Популярные"
                "best" -> "★ Лучшие"
                "rating" -> "★ Рейтинг"
                "homebrew" -> "🛠 Homebrew"
                "misc" -> "📁 Прочее"
                else -> t
            }
            if (seenKeys.add(k)) {
                categories.add(CatalogCategory(k, label))
            }
        }

        // 2. .abc links (active letters only)
        val abcLinks = pagelist.select(".abc a")
        for (a in abcLinks) {
            val href = a.attr("href").trimEnd('/')
            val key = href.substringAfterLast('/').lowercase()
            val text = a.text().trim().uppercase()
            if (key.isNotBlank() && seenKeys.add(key)) {
                categories.add(CatalogCategory(key, text))
            }
        }

        return if (categories.isNotEmpty()) categories else getInitialCategoriesForConsole(consoleSlug)
    }

    /**
     * Scrapes games for a console from emu-land.net/{section}/{slug}/{subpath}/{category}
     * Supports subpaths: roms, iso, games.
     * Handles platforms without /top endpoint, pagination, single-pack containers (Atari), and ISO pages (PS1).
     */
    suspend fun fetchGamesPageForConsole(
        consoleSlug: String,
        consoleName: String,
        section: String = "consoles",
        category: String = "top",
        page: Int = 1
    ): GamesPageResult = withContext(Dispatchers.IO) {
        val games = mutableListOf<GameCard>()
        val subpath = getGameSubpath(consoleSlug)
        val isAll = category.equals("all", ignoreCase = true) || category.isBlank()

        // Build candidate URLs in order of preference
        val candidateUrls = mutableListOf<String>()

        if (!isAll) {
            if (page > 1) {
                candidateUrls.add("$baseUrl/$section/$consoleSlug/$subpath/$category/$page")
            } else {
                candidateUrls.add("$baseUrl/$section/$consoleSlug/$subpath/$category")
            }
            // Fallback: base subpath without category
            if (page > 1) {
                candidateUrls.add("$baseUrl/$section/$consoleSlug/$subpath/$page")
            } else {
                candidateUrls.add("$baseUrl/$section/$consoleSlug/$subpath")
            }
        } else {
            if (page > 1) {
                candidateUrls.add("$baseUrl/$section/$consoleSlug/$subpath/$page")
            } else {
                candidateUrls.add("$baseUrl/$section/$consoleSlug/$subpath")
            }
        }

        var maxPage = page
        var hasNext = false
        var dynamicCategories: List<CatalogCategory>? = null

        for (targetUrl in candidateUrls.distinct()) {
            try {
                val request = Request.Builder()
                    .url(targetUrl)
                    .header("User-Agent", userAgent)
                    .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
                    .header("Accept-Language", "ru-RU,ru;q=0.9,en-US;q=0.8,en;q=0.7")
                    .build()

                val response = client.newCall(request).execute()
                val finalUrl = response.request.url.toString()

                if (response.isSuccessful && !finalUrl.contains("error_404")) {
                    val html = response.body?.string() ?: ""
                    val doc = Jsoup.parse(html, baseUrl)

                    if (dynamicCategories == null) {
                        val extracted = extractCategoriesFromDoc(doc, consoleSlug, subpath)
                        if (extracted.isNotEmpty()) {
                            dynamicCategories = extracted
                        }
                    }

                    // Parse pagination numbers from Emu-Land
                    val pageLinks = doc.select(".num a, div.num a, .pagelist .num a, .pagination a, a.pnum")
                    for (link in pageLinks) {
                        val pNum = link.text().trim().toIntOrNull()
                        if (pNum != null && pNum > maxPage) {
                            maxPage = pNum
                        }
                    }
                    val lastPageLink = doc.selectFirst(".num a[title=Последняя], a[title=Последняя], .num a[title*=последн], a[title*=последн]")
                    if (lastPageLink != null) {
                        val lastHref = lastPageLink.attr("href")
                        val pFromHref = Regex(".*/([0-9]+)").find(lastHref)?.groupValues?.get(1)?.toIntOrNull()
                        if (pFromHref != null && pFromHref > maxPage) {
                            maxPage = pFromHref
                        }
                    }
                    hasNext = doc.select(".num a:contains(Далее), a:contains(Далее), .num a:contains(Next), a:contains(Next)").isNotEmpty() || (maxPage > page)

                    val containers = doc.select(".fcontainer")
                    if (containers.isEmpty()) {
                        continue
                    }

                    for (container in containers) {
                        val headerLink = container.selectFirst(".rheader a")
                            ?: container.selectFirst("a[href*='/$subpath/'], a[href*='/roms/'], a[href*='/games/'], a[href*='/iso/']")

                        val handSpan = container.selectFirst(".rheader .hand, .rheader span[onclick]")
                        val handOnclick = handSpan?.attr("onclick") ?: ""

                        val title = headerLink?.text()?.trim()
                            ?: handSpan?.text()?.trim()
                            ?: container.select("h4.rheader, h4").text().trim()

                        if (title.isBlank()) continue

                        val gamePageHref = headerLink?.attr("href")
                            ?: Regex("goto\\(['\"]([^'\"]+)['\"]").find(handOnclick)?.groupValues?.get(1)
                            ?: ""

                        val gamePageSlug = when {
                            gamePageHref.contains("/$subpath/") -> gamePageHref.substringAfterLast("/$subpath/").substringBefore("?").trim('/')
                            gamePageHref.contains("/roms/") -> gamePageHref.substringAfterLast("/roms/").substringBefore("?").trim('/')
                            gamePageHref.contains("/games/") -> gamePageHref.substringAfterLast("/games/").substringBefore("?").trim('/')
                            gamePageHref.contains("/iso/") -> gamePageHref.substringAfterLast("/iso/").substringBefore("?").trim('/')
                            gamePageHref.isNotBlank() -> gamePageHref.substringAfterLast("/").substringBefore("?").trim('/')
                            else -> ""
                        }

                        val altTitle = container.select(".atitles").text().trim().takeIf { it.isNotEmpty() }

                        val picImg = container.selectFirst(".picture img")
                        val rawCover = picImg?.attr("src")
                        val coverUrl = normalizeImageUrl(rawCover)

                        val screenshotUrls = mutableListOf<String>()
                        val galleryAttr = picImg?.attr("data-gallery")
                        if (!galleryAttr.isNullOrBlank()) {
                            try {
                                val galleryJson = JSONObject(galleryAttr)
                                val listArray = galleryJson.optJSONArray("list")
                                if (listArray != null) {
                                    for (i in 0 until listArray.length()) {
                                        val itemObj = listArray.optJSONObject(i)
                                        val rawSrc = itemObj?.optString("src")
                                        normalizeImageUrl(rawSrc)?.let { normUrl ->
                                            if (normUrl !in screenshotUrls) {
                                                screenshotUrls.add(normUrl)
                                            }
                                        }
                                    }
                                }
                            } catch (e: Exception) {
                                Log.d("EmuLandScraper", "Gallery JSON parse failed: ${e.message}")
                            }
                        }
                        if (screenshotUrls.isEmpty() && coverUrl != null) {
                            screenshotUrls.add(coverUrl)
                        }

                        var genre = "Action"
                        var developer = "Unknown"
                        var year = "N/A"
                        var publisher = "Unknown"

                        container.select(".finfo li").forEach { li ->
                            val text = li.text()
                            when {
                                text.contains("Жанр:", ignoreCase = true) -> genre = text.substringAfter("Жанр:").trim()
                                text.contains("Разработчик:", ignoreCase = true) -> developer = text.substringAfter("Разработчик:").trim()
                                text.contains("Издатель:", ignoreCase = true) -> publisher = text.substringAfter("Издатель:").trim()
                                text.contains("Год выпуска:", ignoreCase = true) -> {
                                    val match = Regex("([12][0-9]{3})").find(text)
                                    if (match != null) year = match.value
                                }
                            }
                        }

                        val desc = container.select(".ftext p").text().trim().ifBlank {
                            container.select(".description p").text().trim()
                        }

                        // Parse file size from .fbottom (e.g. on PS1 and Atari)
                        var fileSize = "ROM"
                        val sizeLi = container.selectFirst(".fbottom li:contains(Размер), .fbottom li:contains(Size), li:contains(Размер)")
                        if (sizeLi != null) {
                            val spanText = sizeLi.selectFirst("span")?.text()?.trim()
                            if (!spanText.isNullOrBlank()) {
                                fileSize = spanText
                            } else {
                                val rawText = sizeLi.text().substringAfter(":").trim()
                                if (rawText.isNotBlank()) fileSize = rawText
                            }
                        }

                        // Extract download IDs and direct file triggers
                        var mfileId: String? = null
                        val dlBtn = container.selectFirst(".btn-sdl, .btn-dl, [onclick*='getmfl'], [onclick*='getfile'], [onclick*='goto_fancy'], [onclick*='game_dl']")
                        val onclick = dlBtn?.attr("onclick") ?: ""

                        val idMatch = Regex("id=([0-9]+)").find(onclick)
                        if (idMatch != null) {
                            mfileId = idMatch.groupValues[1]
                        } else {
                            val elemWithId = container.selectFirst("[id^=text_], [id^=pict_], [id^=mfile_]")
                            val idAttr = elemWithId?.id() ?: ""
                            val fallbackMatch = Regex("([0-9]+)").find(idAttr)
                            if (fallbackMatch != null) {
                                mfileId = fallbackMatch.groupValues[1]
                            } else {
                                val fromHrefMatch = Regex("id=([0-9]+)").find(gamePageHref)
                                if (fromHrefMatch != null) {
                                    mfileId = fromHrefMatch.groupValues[1]
                                }
                            }
                        }

                        val regions = mutableListOf<String>()
                        container.select("img[src*=/flags/]").forEach { flag ->
                            val src = flag.attr("src")
                            when {
                                src.contains("/ru.") -> regions.add("RU")
                                src.contains("/us.") -> regions.add("US")
                                src.contains("/eu.") -> regions.add("EU")
                                src.contains("/jp.") -> regions.add("JP")
                            }
                        }
                        if (regions.isEmpty()) regions.add("US")

                        val uniqueId = mfileId ?: "${consoleSlug}_${gamePageSlug.ifBlank { title.hashCode().toString() }}"
                        val isSingleFile = onclick.contains("act=getfile") || gamePageHref.contains("act=getfile") || (mfileId != null && subpath == "iso")
                        val dlUrl = if (mfileId != null) {
                            if (isSingleFile) {
                                "$baseUrl/$section/$consoleSlug/$subpath?act=getfile&id=$mfileId"
                            } else {
                                "$baseUrl/$section/$consoleSlug/$subpath?act=getmfl&id=$mfileId"
                            }
                        } else if (gamePageHref.isNotBlank()) {
                            if (gamePageHref.startsWith("http")) gamePageHref else "$baseUrl$gamePageHref"
                        } else {
                            "$baseUrl/$section/$consoleSlug/$subpath"
                        }

                        games.add(
                            GameCard(
                                id = uniqueId,
                                consoleSlug = consoleSlug,
                                consoleName = consoleName,
                                section = section,
                                title = title,
                                originalTitle = altTitle,
                                genre = genre,
                                year = year,
                                publisher = if (publisher != "Unknown") publisher else developer,
                                developer = developer,
                                rating = 4.8f,
                                fileSize = fileSize,
                                coverUrl = coverUrl,
                                screenshotUrls = screenshotUrls,
                                description = desc,
                                downloadUrl = dlUrl,
                                mfileId = mfileId,
                                gamePageSlug = gamePageSlug,
                                regions = regions.distinct()
                            )
                        )
                    }

                    if (games.isNotEmpty()) {
                        break
                    }
                }
            } catch (e: Exception) {
                Log.w("EmuLandScraper", "Error fetching games for $consoleSlug at $targetUrl: ${e.message}")
            }
        }

        val finalCategories = dynamicCategories ?: getInitialCategoriesForConsole(consoleSlug)

        if (games.isEmpty() && page == 1) {
            val seeds = getSeedGamesForConsole(consoleSlug, consoleName, section)
            if (seeds.isNotEmpty()) {
                val isTop = category.equals("top", ignoreCase = true)
                val isBest = category.equals("best", ignoreCase = true)
                val filteredSeeds = if (!isAll && !isTop && !isBest && category.isNotBlank()) {
                    seeds.filter { it.title.startsWith(category, ignoreCase = true) }.ifEmpty { seeds }
                } else {
                    seeds
                }
                return@withContext GamesPageResult(
                    games = filteredSeeds,
                    currentPage = 1,
                    totalPages = 1,
                    hasNextPage = false,
                    availableCategories = finalCategories
                )
            }
        }

        GamesPageResult(
            games = games,
            currentPage = page,
            totalPages = maxPage.coerceAtLeast(page),
            hasNextPage = hasNext,
            availableCategories = finalCategories
        )
    }

    suspend fun fetchGamesForConsole(
        consoleSlug: String,
        consoleName: String,
        section: String = "consoles",
        category: String = "top",
        page: Int = 1
    ): List<GameCard> {
        return fetchGamesPageForConsole(consoleSlug, consoleName, section, category, page).games
    }

    /**
     * Extracts mfileId from HTML using high-precision patterns to avoid false matches like movie IDs.
     */
    fun extractMfileIdFromHtml(html: String): String? {
        val patterns = listOf(
            Regex("act=getmfl&(?:amp;)?id=([0-9]+)"),
            Regex("mgame\\(['\"][^'\"]*id=([0-9]+)"),
            Regex("id=[\"']mfile_([0-9]+)[\"']"),
            Regex("id=[\"']text_([0-9]+)[\"']"),
            Regex("id=[\"']pict_([0-9]+)[\"']")
        )
        for (pat in patterns) {
            val match = pat.find(html)
            if (match != null) {
                return match.groupValues[1]
            }
        }
        return null
    }

    /**
     * Classifies ROM version by region or distribution type based on filename and category name.
     */
    private fun classifyRegionOrType(fileName: String, categoryName: String): String {
        return when {
            fileName.contains("[T+Rus", ignoreCase = true) ||
            fileName.contains("Rus", ignoreCase = true) ||
            fileName.contains("(Ru)", ignoreCase = true) ||
            categoryName.contains("Перевед", ignoreCase = true) -> "RUS"

            fileName.contains("(USA)", ignoreCase = true) ||
            fileName.contains("(U)", ignoreCase = true) -> "USA"

            fileName.contains("(Europe)", ignoreCase = true) ||
            fileName.contains("(E)", ignoreCase = true) -> "EUR"

            fileName.contains("(Japan)", ignoreCase = true) ||
            fileName.contains("(J)", ignoreCase = true) -> "JAP"

            fileName.contains("(World)", ignoreCase = true) ||
            fileName.contains("(W)", ignoreCase = true) -> "WLD"

            fileName.contains("Beta", ignoreCase = true) ||
            fileName.contains("Proto", ignoreCase = true) -> "BETA"

            fileName.contains("Hack", ignoreCase = true) ||
            fileName.contains("(H)", ignoreCase = true) -> "HACK"

            categoryName.contains("Пират", ignoreCase = true) ||
            fileName.contains("Unl", ignoreCase = true) ||
            fileName.contains("Pirate", ignoreCase = true) -> "PIRATE"

            categoryName.contains("Good", ignoreCase = true) -> "GoodNES"

            else -> "ROM"
        }
    }

    /**
     * Fetches and parses all ROM file versions for a given mfile ID from Emu-Land's act=getmfl endpoint.
     */
    private fun fetchVersionsFromGetmfl(game: GameCard, mfileId: String): List<RomFileVersion> {
        val versions = mutableListOf<RomFileVersion>()
        val subpath = getGameSubpath(game.consoleSlug)
        val getmflUrl = "$baseUrl/${game.section}/${game.consoleSlug}/$subpath?act=getmfl&id=$mfileId"
        try {
            val req = Request.Builder()
                .url(getmflUrl)
                .header("User-Agent", userAgent)
                .header("Referer", "$baseUrl/${game.section}/${game.consoleSlug}/$subpath")
                .header("X-Requested-With", "XMLHttpRequest")
                .build()

            val resp = client.newCall(req).execute()
            if (resp.isSuccessful) {
                val html = resp.body?.string() ?: ""
                val doc = Jsoup.parse(html, baseUrl)

                val collapsItems = doc.select(".collaps-item")
                if (collapsItems.isNotEmpty()) {
                    for (block in collapsItems) {
                        val rawTitle = block.selectFirst(".title span")?.text()?.trim()
                            ?: block.selectFirst(".title")?.ownText()?.trim()
                            ?: block.selectFirst(".title")?.text()?.trim()
                            ?: "Основные"
                        val categoryName = rawTitle
                            .replace(Regex("\\s*\\(\\?\\)\\s*"), "")
                            .trim()
                            .let { raw ->
                                val parts = raw.split(" ").filter { it.isNotBlank() }
                                if (parts.size >= 2 && parts[0].equals(parts[1], ignoreCase = true)) {
                                    parts.drop(1).joinToString(" ")
                                } else {
                                    raw
                                }
                            }
                            .ifBlank { "Основные" }
                        val items = block.select(".item")
                        for (item in items) {
                            val link = item.selectFirst(".file a, a[href*='act=getmfl'], a[href*='fid=']") ?: continue
                            val fileName = link.text().trim()
                            if (fileName.isBlank()) continue

                            val rawHref = link.attr("href").replace("&amp;", "&")
                            val fid = Regex("fid=([0-9]+)").find(rawHref)?.groupValues?.get(1) ?: fileName.hashCode().toString()
                            val size = item.select(".size, small.size").text().trim().removePrefix("(").removeSuffix(")")

                            val cleanDlUrl = "$baseUrl/${game.section}/${game.consoleSlug}/$subpath?act=getmfl&id=$mfileId&fid=$fid"
                            val regionOrType = classifyRegionOrType(fileName, categoryName)

                            versions.add(
                                RomFileVersion(
                                    fid = fid,
                                    name = fileName,
                                    size = size.ifBlank { "ROM" },
                                    category = categoryName,
                                    downloadUrl = cleanDlUrl,
                                    regionOrType = regionOrType
                                )
                            )
                        }
                    }
                } else {
                    val links = doc.select("a[href*='act=getmfl'][href*='fid=']")
                    for (link in links) {
                        val fileName = link.text().trim()
                        if (fileName.isBlank()) continue
                        val rawHref = link.attr("href").replace("&amp;", "&")
                        val fid = Regex("fid=([0-9]+)").find(rawHref)?.groupValues?.get(1) ?: fileName.hashCode().toString()
                        val cleanDlUrl = "$baseUrl/${game.section}/${game.consoleSlug}/$subpath?act=getmfl&id=$mfileId&fid=$fid"
                        val regionOrType = classifyRegionOrType(fileName, "Основные")
                        versions.add(
                            RomFileVersion(
                                fid = fid,
                                name = fileName,
                                size = "ROM",
                                category = "Основные",
                                downloadUrl = cleanDlUrl,
                                regionOrType = regionOrType
                            )
                        )
                    }
                }
            }
        } catch (e: Exception) {
            Log.e("EmuLandScraper", "Error in fetchVersionsFromGetmfl for ${game.title} (mfileId=$mfileId): ${e.message}", e)
        }
        return versions
    }

    /**
     * Fetches all available ROM versions/files (Regions, Translations, Hacks, Prototypes) for a game.
     */
    suspend fun fetchRomVersions(game: GameCard): List<RomFileVersion> = withContext(Dispatchers.IO) {
        val versions = mutableListOf<RomFileVersion>()
        val subpath = getGameSubpath(game.consoleSlug)
        var mfileId = game.mfileId

        // Handle direct single-file games (PS1 ISOs, Atari ROM packs with act=getfile)
        val isSingleFile = game.downloadUrl.contains("act=getfile") || (mfileId != null && subpath == "iso")
        if (isSingleFile) {
            val fileId = mfileId ?: Regex("id=([0-9]+)").find(game.downloadUrl)?.groupValues?.get(1) ?: "1"
            val directFileUrl = if (game.downloadUrl.contains("act=getfile")) {
                game.downloadUrl
            } else {
                "$baseUrl/${game.section}/${game.consoleSlug}/$subpath?act=getfile&id=$fileId"
            }
            val ext = if (subpath == "iso") "7z" else "zip"
            versions.add(
                RomFileVersion(
                    fid = fileId,
                    name = if (game.title.endsWith(".$ext", ignoreCase = true)) game.title else "${game.title}.$ext",
                    size = game.fileSize.ifBlank { "Original" },
                    category = "Основные",
                    downloadUrl = directFileUrl,
                    regionOrType = classifyRegionOrType(game.title, "Основные")
                )
            )
            return@withContext versions
        }

        // 1. If mfileId is missing or non-numeric, resolve from downloadUrl or game page
        if (mfileId.isNullOrBlank() || !mfileId.all { it.isDigit() }) {
            val idFromDl = Regex("act=(?:getmfl|getfile)&(?:amp;)?id=([0-9]+)").find(game.downloadUrl)?.groupValues?.get(1)
            if (!idFromDl.isNullOrBlank()) {
                mfileId = idFromDl
            } else {
                val pageSlug = game.gamePageSlug?.takeIf { it.isNotBlank() }
                    ?: game.downloadUrl.substringAfterLast("/$subpath/").substringBefore("?").trim('/').takeIf { it.isNotBlank() }
                    ?: game.downloadUrl.substringAfterLast("/roms/").substringBefore("?").trim('/').takeIf { it.isNotBlank() }
                if (pageSlug != null) {
                    mfileId = fetchMfileIdFromGamePage(game.section, game.consoleSlug, pageSlug)
                }
            }
        }

        // 2. Try fetching versions using mfileId
        if (!mfileId.isNullOrBlank()) {
            val fetched = fetchVersionsFromGetmfl(game, mfileId)
            if (fetched.isNotEmpty()) {
                versions.addAll(fetched)
            }
        }

        // 3. If mfileId failed (e.g. was obsolete or 404), fetch fresh game page and re-extract
        if (versions.isEmpty()) {
            val pageSlug = game.gamePageSlug?.takeIf { it.isNotBlank() }
                ?: game.downloadUrl.substringAfterLast("/$subpath/").substringBefore("?").trim('/').takeIf { it.isNotBlank() }
                ?: game.downloadUrl.substringAfterLast("/roms/").substringBefore("?").trim('/').takeIf { it.isNotBlank() }

            if (pageSlug != null) {
                val freshId = fetchMfileIdFromGamePage(game.section, game.consoleSlug, pageSlug)
                if (!freshId.isNullOrBlank() && freshId != mfileId) {
                    mfileId = freshId
                    val fetched = fetchVersionsFromGetmfl(game, freshId)
                    if (fetched.isNotEmpty()) {
                        versions.addAll(fetched)
                    }
                }
            }
        }

        // 4. Fallback only if network truly returned nothing
        if (versions.isEmpty()) {
            val fallbackDlUrl = if (!mfileId.isNullOrBlank()) {
                "$baseUrl/${game.section}/${game.consoleSlug}/$subpath?act=getmfl&id=$mfileId"
            } else {
                game.downloadUrl
            }
            val ext = if (subpath == "iso") "7z" else "zip"
            versions.add(
                RomFileVersion(
                    fid = mfileId ?: "default",
                    name = if (game.title.endsWith(".$ext", ignoreCase = true)) game.title else "${game.title}.$ext",
                    size = game.fileSize.ifBlank { "Standard" },
                    category = "Основные",
                    downloadUrl = fallbackDlUrl,
                    regionOrType = classifyRegionOrType(game.title, "USA")
                )
            )
        }

        versions
    }

    suspend fun resolveDirectDownloadFromUrl(fileUrl: String, referer: String): String? = withContext(Dispatchers.IO) {
        try {
            val cleanUrl = fileUrl.replace("&amp;", "&")
            val req = Request.Builder()
                .url(cleanUrl)
                .header("User-Agent", userAgent)
                .header("Referer", referer)
                .build()

            val resp = noRedirectClient.newCall(req).execute()
            val locationHeader = resp.header("Location")
            if (!locationHeader.isNullOrBlank()) {
                val finalUrl = when {
                    locationHeader.startsWith("//") -> "https:$locationHeader"
                    locationHeader.startsWith("/") -> "$baseUrl$locationHeader"
                    else -> locationHeader
                }
                return@withContext finalUrl.replace(" ", "%20")
            }
        } catch (e: Exception) {
            Log.e("EmuLandScraper", "Error resolving direct download from url $fileUrl: ${e.message}", e)
        }
        null
    }

    /**
     * Resolves the direct downloadable ZIP/7Z URL using emu-land's redirect endpoint (getmfl or getfile).
     */
    suspend fun resolveDirectDownloadUrl(game: GameCard): String? = withContext(Dispatchers.IO) {
        val versions = fetchRomVersions(game)
        val preferredVersion = versions.firstOrNull { it.regionOrType == "RUS" }
            ?: versions.firstOrNull { it.regionOrType == "USA" }
            ?: versions.firstOrNull()

        if (preferredVersion != null) {
            val subpath = getGameSubpath(game.consoleSlug)
            val referer = "$baseUrl/${game.section}/${game.consoleSlug}/$subpath"
            val resolved = resolveDirectDownloadFromUrl(preferredVersion.downloadUrl, referer)
            if (resolved != null) return@withContext resolved
        }

        null
    }

    private fun fetchMfileIdFromGamePage(section: String, consoleSlug: String, gamePageSlug: String): String? {
        try {
            val subpath = getGameSubpath(consoleSlug)
            val url = "$baseUrl/$section/$consoleSlug/$subpath/$gamePageSlug"
            val req = Request.Builder().url(url).header("User-Agent", userAgent).build()
            val resp = client.newCall(req).execute()
            if (resp.isSuccessful) {
                val html = resp.body?.string() ?: ""
                return extractMfileIdFromHtml(html)
            }
        } catch (_: Exception) {}
        return null
    }

    /**
     * Scrapes full game card (description, extra screenshots, specs) from game page
     */
    suspend fun fetchGameCardDetails(game: GameCard): GameCard = withContext(Dispatchers.IO) {
        if (game.screenshotUrls.size > 2 && game.description.length > 100 && !game.mfileId.isNullOrBlank()) {
            return@withContext game
        }

        val gameSlug = game.gamePageSlug ?: return@withContext game
        val subpath = getGameSubpath(game.consoleSlug)
        val url = "$baseUrl/${game.section}/${game.consoleSlug}/$subpath/$gameSlug"

        try {
            val request = Request.Builder()
                .url(url)
                .header("User-Agent", userAgent)
                .build()

            val response = client.newCall(request).execute()
            if (response.isSuccessful) {
                val html = response.body?.string() ?: ""
                val doc = Jsoup.parse(html, baseUrl)

                val screens = mutableListOf<String>()
                screens.addAll(game.screenshotUrls)

                doc.select(".ss-area a.ss, .picture img").forEach { elem ->
                    val rawSrc = if (elem.hasAttr("href")) elem.attr("href") else elem.attr("src")
                    normalizeImageUrl(rawSrc)?.let { norm ->
                        if (norm !in screens) screens.add(norm)
                    }
                }

                val descElem = doc.selectFirst(".ftext p, .description p, #description")
                val desc = descElem?.text()?.trim() ?: game.description
                val resolvedMfileId = extractMfileIdFromHtml(html) ?: game.mfileId

                return@withContext game.copy(
                    mfileId = resolvedMfileId,
                    screenshotUrls = if (screens.isNotEmpty()) screens else game.screenshotUrls,
                    description = if (desc.isNotEmpty()) desc else game.description,
                    coverUrl = screens.firstOrNull() ?: game.coverUrl
                )
            }
        } catch (e: Exception) {
            Log.w("EmuLandScraper", "Error fetching game details for ${game.title}: ${e.message}")
        }

        game
    }

    /**
     * Global site-wide search across ALL platforms and consoles from Emu-Land.net
     */
    suspend fun searchGamesSiteWide(query: String): List<GameCard> = withContext(Dispatchers.IO) {
        val results = mutableListOf<GameCard>()
        val cleanQuery = query.trim()
        if (cleanQuery.isBlank()) return@withContext emptyList()

        try {
            val encoded = java.net.URLEncoder.encode(cleanQuery, "UTF-8")
            val searchUrl = "$baseUrl/search_games?q=$encoded&id=all"
            val req = Request.Builder()
                .url(searchUrl)
                .header("User-Agent", userAgent)
                .header("Referer", baseUrl)
                .build()

            val resp = client.newCall(req).execute()
            if (resp.isSuccessful) {
                val html = resp.body?.string() ?: ""
                val doc = Jsoup.parse(html, baseUrl)

                val paragraphs = doc.select("p:has(a[href*='/roms/'], a[href*='/games/'], a[href*='/iso/'])")
                for (p in paragraphs) {
                    val link = p.selectFirst("a[href*='/roms/'], a[href*='/games/'], a[href*='/iso/']") ?: continue
                    val href = link.attr("href")
                    val title = link.text().trim()
                    if (title.isBlank()) continue

                    val parts = href.trim('/').split('/')
                    if (parts.size < 4) continue
                    val section = parts[0]
                    val consoleSlug = parts[1]
                    val subFolder = parts[2]
                    val gamePageSlug = parts[3]

                    val imgTag = p.selectFirst("img")
                    val coverSrc = imgTag?.attr("src")?.let {
                        when {
                            it.startsWith("//") -> "https:$it"
                            it.startsWith("/") -> "$baseUrl$it"
                            else -> it
                        }
                    }

                    val smalls = p.select("small.muted-text")
                    var genre = "Retro Game"
                    for (sm in smalls) {
                        val txt = sm.text().trim().removePrefix("|").trim()
                        if (!txt.contains("Игроки", ignoreCase = true) && !txt.contains("Players", ignoreCase = true)) {
                            genre = txt
                            break
                        }
                    }

                    val consoleName = mapSlugToConsoleInfo(consoleSlug, consoleSlug.uppercase(), section, 0).name

                    val id = "${consoleSlug}_$gamePageSlug"
                    results.add(
                        GameCard(
                            id = id,
                            consoleSlug = consoleSlug,
                            consoleName = consoleName,
                            section = section,
                            title = title,
                            originalTitle = title,
                            genre = genre,
                            year = "N/A",
                            rating = 4.8f,
                            fileSize = "ROM",
                            coverUrl = coverSrc,
                            description = "",
                            downloadUrl = "$baseUrl/$section/$consoleSlug/$subFolder/$gamePageSlug",
                            gamePageSlug = gamePageSlug,
                            regions = listOf("USA")
                        )
                    )
                }
            }
        } catch (e: Exception) {
            Log.e("EmuLandScraper", "Error searching games site-wide for '$query': ${e.message}", e)
        }

        results
    }

    private fun mapSlugToConsoleInfo(slug: String, rawTitle: String, section: String, index: Int): ConsoleInfo {
        val (name, shortName, cat, folder, year) = when (slug) {
            "dendy" -> ConsoleConfig("NES / Famicom / Dendy", "Dendy / NES", "8-bit", "Dendy (NES)", "1983")
            "genesis" -> ConsoleConfig("Sega Mega Drive / Genesis", "Mega Drive", "16-bit", "Sega Mega Drive", "1988")
            "snes" -> ConsoleConfig("Super Nintendo (SNES)", "SNES", "16-bit", "Super Nintendo", "1990")
            "gba" -> ConsoleConfig("Game Boy Advance", "GBA", "Handheld", "Game Boy Advance", "2001")
            "gb" -> ConsoleConfig("Game Boy", "Game Boy", "Handheld", "Game Boy", "1989")
            "gbc" -> ConsoleConfig("Game Boy Color", "GBC", "Handheld", "Game Boy Color", "1998")
            "psx" -> ConsoleConfig("Sony PlayStation 1", "PS1", "32-bit", "PlayStation 1", "1994")
            "n64" -> ConsoleConfig("Nintendo 64", "N64", "64-bit", "Nintendo 64", "1996")
            "sms" -> ConsoleConfig("Sega Master System", "Master System", "8-bit", "Sega Master System", "1985")
            "32x" -> ConsoleConfig("Sega 32X", "Sega 32X", "32-bit", "Sega 32X", "1994")
            "segacd" -> ConsoleConfig("Sega CD / Mega CD", "Sega CD", "16-bit CD", "Sega CD", "1991")
            "sg-1000" -> ConsoleConfig("Sega SG-1000", "SG-1000", "8-bit", "Sega SG-1000", "1983")
            "pce" -> ConsoleConfig("PC Engine / TurboGrafx-16", "PC Engine", "16-bit", "PC Engine", "1987")
            "pcecd" -> ConsoleConfig("PC Engine CD / TurboGrafx CD", "PC Engine CD", "16-bit CD", "PC Engine CD", "1988")
            "3do" -> ConsoleConfig("3DO Interactive", "3DO", "32-bit", "3DO", "1993")
            "famicom_disk_system" -> ConsoleConfig("Famicom Disk System", "Famicom Disk", "8-bit", "Famicom Disk System", "1986")
            "neogeocd" -> ConsoleConfig("Neo Geo CD", "Neo Geo CD", "16-bit CD", "Neo Geo CD", "1994")
            "2600" -> ConsoleConfig("Atari 2600", "Atari 2600", "Classic", "Atari 2600", "1977")
            "5200" -> ConsoleConfig("Atari 5200", "Atari 5200", "Classic", "Atari 5200", "1982")
            "7800" -> ConsoleConfig("Atari 7800", "Atari 7800", "Classic", "Atari 7800", "1986")
            "jaguar" -> ConsoleConfig("Atari Jaguar", "Jaguar", "64-bit", "Atari Jaguar", "1993")
            "coleco" -> ConsoleConfig("ColecoVision", "ColecoVision", "Classic", "ColecoVision", "1982")
            "vectrex" -> ConsoleConfig("Vectrex", "Vectrex", "Classic", "Vectrex", "1982")
            "intellivision" -> ConsoleConfig("Intellivision", "Intellivision", "Classic", "Intellivision", "1979")
            "arcadia" -> ConsoleConfig("Emerson Arcadia 2001", "Arcadia", "Classic", "Arcadia", "1982")
            "chaf" -> ConsoleConfig("Fairchild Channel F", "Channel F", "Classic", "Channel F", "1976")
            "gg" -> ConsoleConfig("Sega Game Gear", "Game Gear", "Handheld", "Game Gear", "1990")
            "lynx" -> ConsoleConfig("Atari Lynx", "Lynx", "Handheld", "Atari Lynx", "1989")
            "ngp" -> ConsoleConfig("Neo Geo Pocket", "NGP", "Handheld", "Neo Geo Pocket", "1998")
            "ws" -> ConsoleConfig("Bandai WonderSwan", "WonderSwan", "Handheld", "WonderSwan", "1999")
            "vboy" -> ConsoleConfig("Nintendo Virtual Boy", "Virtual Boy", "Handheld", "Virtual Boy", "1995")
            "pmini" -> ConsoleConfig("Pokemon Mini", "Pokemon Mini", "Handheld", "Pokemon Mini", "2001")
            "sv" -> ConsoleConfig("Watara Supervision", "Supervision", "Handheld", "Supervision", "1992")
            else -> ConsoleConfig(rawTitle, rawTitle, if (section == "portable") "Handheld" else "Console", rawTitle.replace("/", "_"), "")
        }

        return ConsoleInfo(
            slug = slug,
            name = name,
            shortName = shortName,
            category = cat,
            folderName = folder,
            section = section,
            isEnabled = true,
            order = index,
            releaseYear = year
        )
    }

    private data class ConsoleConfig(
        val name: String,
        val shortName: String,
        val category: String,
        val folder: String,
        val year: String
    )

    fun getDefaultConsoles(): List<ConsoleInfo> {
        return listOf(
            ConsoleInfo("dendy", "NES / Famicom / Dendy", "Dendy / NES", "8-bit", "Dendy (NES)", "consoles", true, 0, "1983", "gamepad", "2,400+ ROMs"),
            ConsoleInfo("genesis", "Sega Mega Drive / Genesis", "Mega Drive", "16-bit", "Sega Mega Drive", "consoles", true, 1, "1988", "gamepad", "1,800+ ROMs"),
            ConsoleInfo("snes", "Super Nintendo (SNES)", "SNES", "16-bit", "Super Nintendo", "consoles", true, 2, "1990", "gamepad", "2,100+ ROMs"),
            ConsoleInfo("gba", "Game Boy Advance", "GBA", "Handheld", "Game Boy Advance", "portable", true, 3, "2001", "gamepad", "1,950+ ROMs"),
            ConsoleInfo("psx", "Sony PlayStation 1", "PlayStation 1", "32-bit", "PlayStation 1", "consoles", true, 4, "1994", "gamepad", "3,200+ ISO"),
            ConsoleInfo("n64", "Nintendo 64", "N64", "64-bit", "Nintendo 64", "consoles", true, 5, "1996", "gamepad", "380+ ROMs"),
            ConsoleInfo("gb", "Game Boy", "Game Boy", "Handheld", "Game Boy", "portable", true, 6, "1989", "gamepad", "1,600+ ROMs"),
            ConsoleInfo("gbc", "Game Boy Color", "Game Boy Color", "Handheld", "Game Boy Color", "portable", true, 7, "1998", "gamepad", "1,100+ ROMs"),
            ConsoleInfo("32x", "Sega 32X", "Sega 32X", "32-bit", "Sega 32X", "consoles", true, 8, "1994", "gamepad", "40+ ROMs"),
            ConsoleInfo("segacd", "Sega CD / Mega CD", "Sega CD", "16-bit CD", "Sega CD", "consoles", true, 9, "1991", "gamepad", "150+ ROMs"),
            ConsoleInfo("pce", "PC Engine / TurboGrafx-16", "PC Engine", "16-bit", "PC Engine", "consoles", true, 10, "1987", "gamepad", "650+ ROMs"),
            ConsoleInfo("pcecd", "PC Engine CD / TurboGrafx CD", "PC Engine CD", "16-bit CD", "PC Engine CD", "consoles", true, 11, "1988", "gamepad", "180+ ROMs"),
            ConsoleInfo("3do", "3DO Interactive Multiplayer", "3DO", "32-bit", "3DO", "consoles", true, 12, "1993", "gamepad", "280+ ROMs"),
            ConsoleInfo("sms", "Sega Master System", "Master System", "8-bit", "Sega Master System", "consoles", true, 13, "1985", "gamepad", "420+ ROMs"),
            ConsoleInfo("famicom_disk_system", "Famicom Disk System", "Famicom Disk", "8-bit", "Famicom Disk System", "consoles", true, 14, "1986", "gamepad", "190+ ROMs"),
            ConsoleInfo("neogeocd", "Neo Geo CD", "Neo Geo CD", "16-bit CD", "Neo Geo CD", "consoles", true, 15, "1994", "gamepad", "100+ ROMs"),
            ConsoleInfo("2600", "Atari 2600", "Atari 2600", "Classic", "Atari 2600", "consoles", true, 16, "1977", "gamepad", "7,200+ ROMs"),
            ConsoleInfo("5200", "Atari 5200", "Atari 5200", "Classic", "Atari 5200", "consoles", true, 17, "1982", "gamepad", "200+ ROMs"),
            ConsoleInfo("7800", "Atari 7800", "Atari 7800", "Classic", "Atari 7800", "consoles", true, 18, "1986", "gamepad", "120+ ROMs"),
            ConsoleInfo("jaguar", "Atari Jaguar", "Jaguar", "64-bit", "Atari Jaguar", "consoles", true, 19, "1993", "gamepad", "80+ ROMs"),
            ConsoleInfo("gg", "Sega Game Gear", "Game Gear", "Handheld", "Game Gear", "portable", true, 20, "1990", "gamepad", "390+ ROMs"),
            ConsoleInfo("lynx", "Atari Lynx", "Lynx", "Handheld", "Atari Lynx", "portable", true, 21, "1989", "gamepad", "80+ ROMs"),
            ConsoleInfo("ngp", "Neo Geo Pocket", "NGP", "Handheld", "Neo Geo Pocket", "portable", true, 22, "1998", "gamepad", "80+ ROMs"),
            ConsoleInfo("ws", "Bandai WonderSwan", "WonderSwan", "Handheld", "WonderSwan", "portable", true, 23, "1999", "gamepad", "110+ ROMs")
        )
    }

    fun getSeedGamesForConsole(slug: String, consoleName: String, section: String = "consoles"): List<GameCard> {
        return when (slug) {
            "dendy" -> listOf(
                GameCard(
                    id = "24471",
                    consoleSlug = "dendy",
                    consoleName = consoleName,
                    section = section,
                    title = "Battletoads & Double Dragon",
                    originalTitle = "The Ultimate Team",
                    genre = "beat 'em up",
                    year = "1993",
                    publisher = "Tradewest",
                    developer = "Rare Limited",
                    rating = 5.0f,
                    fileSize = "256 KB",
                    coverUrl = "https://ss.emu-land.net/nes_pict/Battletoads%20%26%20Double%20Dragon_03.png",
                    screenshotUrls = listOf(
                        "https://ss.emu-land.net/nes_pict/Battletoads%20%26%20Double%20Dragon_03.png",
                        "https://ss.emu-land.net/nes_pict/Battletoads%20%26%20Double%20Dragon_00.png",
                        "https://ss.emu-land.net/nes_pict/Battletoads%20%26%20Double%20Dragon_05.png"
                    ),
                    description = "Слияние двух великих игр Battletoads и Double Dragon. Пять бойцов на выбор, кооператив на двоих игроков и динамичные уровни.",
                    downloadUrl = "https://www.emu-land.net/consoles/dendy/roms?act=getmfl&id=24471",
                    mfileId = "24471",
                    gamePageSlug = "battletoads-and-double-dragon-the-ultimate-team",
                    regions = listOf("US", "RU", "EU")
                ),
                GameCard(
                    id = "24572",
                    consoleSlug = "dendy",
                    consoleName = consoleName,
                    section = section,
                    title = "Chip 'n Dale: Rescue Rangers",
                    originalTitle = "Chip to Dale no Daisakusen",
                    genre = "platform",
                    year = "1990",
                    publisher = "Capcom Co., Ltd.",
                    developer = "Capcom",
                    rating = 4.9f,
                    fileSize = "128 KB",
                    coverUrl = "https://ss.emu-land.net/nes_pict/Chip%20%27n%20Dale%20Rescue%20Rangers_02.png",
                    screenshotUrls = listOf(
                        "https://ss.emu-land.net/nes_pict/Chip%20%27n%20Dale%20Rescue%20Rangers_02.png",
                        "https://ss.emu-land.net/nes_pict/Chip%20%27n%20Dale%20Rescue%20Rangers_00.png",
                        "https://ss.emu-land.net/nes_pict/Chip%20%27n%20Dale%20Rescue%20Rangers_04.png"
                    ),
                    description = "Чип, Дейл и их друзья снова в деле! Один из лучших диснеевских платформеров на двоих игроков.",
                    downloadUrl = "https://www.emu-land.net/consoles/dendy/roms?act=getmfl&id=24572",
                    mfileId = "24572",
                    gamePageSlug = "chip-n-dale-rescue-rangers",
                    regions = listOf("US", "RU", "JP")
                ),
                GameCard(
                    id = "25812",
                    consoleSlug = "dendy",
                    consoleName = consoleName,
                    section = section,
                    title = "Teenage Mutant Ninja Turtles",
                    originalTitle = "Gekikame Ninja Den",
                    genre = "platform / action",
                    year = "1989",
                    publisher = "Konami / Ultra Games",
                    developer = "Konami",
                    rating = 4.8f,
                    fileSize = "138 KB",
                    coverUrl = "https://ss.emu-land.net/nes_pict/Teenage%20Mutant%20Ninja%20Turtles_02.png",
                    screenshotUrls = listOf(
                        "https://ss.emu-land.net/nes_pict/Teenage%20Mutant%20Ninja%20Turtles_02.png",
                        "https://ss.emu-land.net/nes_pict/Teenage%20Mutant%20Ninja%20Turtles_00.png"
                    ),
                    description = "Черепашки-ниндзя исследуют улицы Нью-Йорка и канализацию, спасая Эйприл О'Нил и Сплинтера от Шреддера.",
                    downloadUrl = "https://www.emu-land.net/consoles/dendy/roms?act=getmfl&id=25812",
                    mfileId = "25812",
                    gamePageSlug = "teenage-mutant-ninja-turtles",
                    regions = listOf("US", "RU", "EU", "JP")
                )
            )

            "2600" -> listOf(
                GameCard(
                    id = "3906",
                    consoleSlug = "2600",
                    consoleName = consoleName,
                    section = section,
                    title = "Good2600 V3.14 (Full ROM Pack)",
                    originalTitle = "Good2600 V3.14",
                    genre = "Collection / Classic",
                    year = "1982",
                    publisher = "Atari / Various",
                    developer = "Various",
                    rating = 4.9f,
                    fileSize = "4.9 MiB",
                    coverUrl = "https://ss.emu-land.net/nes_pict/Battletoads%20%26%20Double%20Dragon_03.png",
                    screenshotUrls = listOf("https://ss.emu-land.net/nes_pict/Battletoads%20%26%20Double%20Dragon_03.png"),
                    description = "Полный сборник всех 7,200 РОМов для легендарной игровой консоли Atari 2600 по сету Good2600 V3.14.",
                    downloadUrl = "$baseUrl/consoles/2600/roms?act=getfile&id=3906",
                    mfileId = "3906",
                    gamePageSlug = "good2600",
                    regions = listOf("US", "EU")
                ),
                GameCard(
                    id = "at_pitfall",
                    consoleSlug = "2600",
                    consoleName = consoleName,
                    section = section,
                    title = "Pitfall!",
                    originalTitle = "Pitfall!",
                    genre = "Platformer / Adventure",
                    year = "1982",
                    publisher = "Activision",
                    developer = "David Crane",
                    rating = 4.9f,
                    fileSize = "4 KB",
                    coverUrl = "https://ss.emu-land.net/nes_pict/Chip%20%27n%20Dale%20Rescue%20Rangers_02.png",
                    screenshotUrls = listOf("https://ss.emu-land.net/nes_pict/Chip%20%27n%20Dale%20Rescue%20Rangers_02.png"),
                    description = "Один из величайших платформеров в истории: преодолевайте крокодилов, змей и зыбучие пески джунглей.",
                    downloadUrl = "$baseUrl/consoles/2600/roms",
                    regions = listOf("US")
                ),
                GameCard(
                    id = "at_river_raid",
                    consoleSlug = "2600",
                    consoleName = consoleName,
                    section = section,
                    title = "River Raid",
                    originalTitle = "River Raid",
                    genre = "Scrolling Shooter",
                    year = "1982",
                    publisher = "Activision",
                    developer = "Carol Shaw",
                    rating = 4.9f,
                    fileSize = "8 KB",
                    coverUrl = "https://ss.emu-land.net/genesis_pict/OutRun_02.png",
                    screenshotUrls = listOf("https://ss.emu-land.net/genesis_pict/OutRun_02.png"),
                    description = "Нестареющая классика вертикального скролл-шутера вдоль извилистой реки с уничтожением мостов и дозаправкой.",
                    downloadUrl = "$baseUrl/consoles/2600/roms",
                    regions = listOf("US")
                )
            )

            "32x" -> listOf(
                GameCard(
                    id = "32x_knuckles",
                    consoleSlug = "32x",
                    consoleName = consoleName,
                    section = section,
                    title = "Knuckles' Chaotix",
                    originalTitle = "Chaotix",
                    genre = "Platformer",
                    year = "1995",
                    publisher = "Sega",
                    developer = "Sonic Team",
                    rating = 4.8f,
                    fileSize = "3.2 MB",
                    coverUrl = "https://ss.emu-land.net/genesis_pict/Sonic%20the%20Hedgehog_02.png",
                    screenshotUrls = listOf("https://ss.emu-land.net/genesis_pict/Sonic%20the%20Hedgehog_02.png"),
                    description = "Эксклюзивный платформер серии Sonic для 32X с эластичной резиновой связкой между двумя персонажами.",
                    downloadUrl = "$baseUrl/consoles/32x/roms",
                    regions = listOf("US", "EU", "JP")
                )
            )

            "psx" -> listOf(
                GameCard(
                    id = "16407",
                    consoleSlug = "psx",
                    consoleName = consoleName,
                    section = section,
                    title = "The Adventure of Little Ralph",
                    originalTitle = "Chippoke Ralph no Daibouken (J)",
                    genre = "Platformer / Action",
                    year = "1999",
                    publisher = "New",
                    developer = "New",
                    rating = 4.9f,
                    fileSize = "25.2 MiB",
                    coverUrl = "//ss.emu-land.net/psx_pict/Adventure_of_Little_Ralph_(PSX)_0037.png",
                    screenshotUrls = listOf("//ss.emu-land.net/psx_pict/Adventure_of_Little_Ralph_(PSX)_0037.png"),
                    description = "Великолепный платформер-приключение для PlayStation! Настоящий праздник для ценителей классического 2D-геймплея.",
                    downloadUrl = "$baseUrl/consoles/psx/iso?act=getfile&id=16407",
                    mfileId = "16407",
                    gamePageSlug = "adventure-of-little-ralph",
                    regions = listOf("JP")
                ),
                GameCard(
                    id = "16408",
                    consoleSlug = "psx",
                    consoleName = consoleName,
                    section = section,
                    title = "Cotton Original, Fantastic Night Dreams",
                    originalTitle = "Cotton Original",
                    genre = "Shoot 'em up",
                    year = "1999",
                    publisher = "Success",
                    developer = "Success",
                    rating = 4.8f,
                    fileSize = "28.5 MiB",
                    coverUrl = "//ss.emu-land.net/psx_pict/Cotton_Original_(PSX)_0000.png",
                    screenshotUrls = listOf("//ss.emu-land.net/psx_pict/Cotton_Original_(PSX)_0000.png"),
                    description = "Переиздание культовой аркадной ведьмочки Коттон с улучшенным балансом, сочной графикой и музыкой.",
                    downloadUrl = "$baseUrl/consoles/psx/iso?act=getfile&id=16408",
                    mfileId = "16408",
                    gamePageSlug = "cotton-original",
                    regions = listOf("JP")
                )
            )

            else -> emptyList()
        }
    }
}
