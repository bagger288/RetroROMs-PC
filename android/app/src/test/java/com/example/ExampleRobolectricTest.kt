package com.example

import android.content.Context
import androidx.room.Room
import androidx.test.core.app.ApplicationProvider
import com.example.data.local.AppDatabase
import com.example.data.local.ConsoleEntity
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [36])
class ExampleRobolectricTest {

  @Test
  fun `read string from context`() {
    val context = ApplicationProvider.getApplicationContext<Context>()
    val appName = context.getString(R.string.app_name)
    assertEquals("RetroROMs", appName)
  }

  @Test
  fun `console reordering and move to top updates sort order`() = runBlocking {
    val context = ApplicationProvider.getApplicationContext<Context>()
    val db = Room.inMemoryDatabaseBuilder(context, AppDatabase::class.java)
        .allowMainThreadQueries()
        .build()
    val dao = db.consoleDao()

    val c1 = ConsoleEntity(slug = "nes", name = "NES", shortName = "NES", category = "8-bit", folderName = "NES", sortOrder = 0)
    val c2 = ConsoleEntity(slug = "snes", name = "SNES", shortName = "SNES", category = "16-bit", folderName = "SNES", sortOrder = 1)
    val c3 = ConsoleEntity(slug = "genesis", name = "Genesis", shortName = "Genesis", category = "16-bit", folderName = "Genesis", sortOrder = 2)
    dao.insertConsoles(listOf(c1, c2, c3))

    // Move c3 (Genesis) to top
    dao.moveConsoleToTop("genesis")
    val listAfterTop = dao.getAllConsolesList()
    assertEquals("genesis", listAfterTop[0].slug)
    assertEquals(0, listAfterTop[0].sortOrder)
    assertEquals("nes", listAfterTop[1].slug)
    assertEquals("snes", listAfterTop[2].slug)

    // Swap position 1 and 2
    dao.swapConsoleOrder("nes", "snes")
    val listAfterSwap = dao.getAllConsolesList()
    assertEquals("genesis", listAfterSwap[0].slug)
    assertEquals("snes", listAfterSwap[1].slug)
    assertEquals("nes", listAfterSwap[2].slug)

    db.close()
  }
}

