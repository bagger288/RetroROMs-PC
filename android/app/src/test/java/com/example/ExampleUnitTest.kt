package com.example

import org.junit.Assert.*
import org.junit.Test

/**
 * Example local unit test, which will execute on the development machine (host).
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
class ExampleUnitTest {
  @Test
  fun addition_isCorrect() {
    assertEquals(4, 2 + 2)
  }

  @Test
  fun unsupportedConsolesWithoutGamesAreExcluded() {
    val unsupported = setOf("nds", "3ds", "nintendo-3ds", "wii", "wii_u", "xbox", "xbox360", "ps2", "ps3", "psp", "dreamcast", "gamecube", "saturn")
    for (slug in unsupported) {
      assertFalse("Console $slug should not be in CONSOLES_WITH_GAMES", com.example.data.remote.EmuLandScraper.CONSOLES_WITH_GAMES.contains(slug))
    }
  }

  @Test
  fun consolesWithSinglePageReturnAllCategory() {
    val scraper = com.example.data.remote.EmuLandScraper()
    val psxCats = scraper.getInitialCategoriesForConsole("psx")
    assertEquals(1, psxCats.size)
    assertEquals("all", psxCats[0].key)

    val atariCats = scraper.getInitialCategoriesForConsole("2600")
    assertEquals(1, atariCats.size)
    assertEquals("all", atariCats[0].key)
  }

  @Test
  fun consolesWithoutTopCategoryDoNotHaveTop() {
    val scraper = com.example.data.remote.EmuLandScraper()
    val neogeoCats = scraper.getInitialCategoriesForConsole("neogeocd")
    assertFalse("Neo Geo CD should not have top category", neogeoCats.any { it.key == "top" })
    assertTrue("Neo Geo CD should have best category", neogeoCats.any { it.key == "best" })

    val jaguarCats = scraper.getInitialCategoriesForConsole("jaguar")
    assertFalse("Jaguar should not have top category", jaguarCats.any { it.key == "top" })
    assertFalse("Jaguar should not have best category", jaguarCats.any { it.key == "best" })
  }
}
