import { mkdir } from 'node:fs/promises'
import { join } from 'node:path'

/**
 * Vitest setup file that runs before all tests
 * Creates necessary directories for test execution
 */
export async function setup() {
  const testDir = join(__dirname)
  const cssDir = join(testDir, '.css')
  
  try {
    // Create .css directory if it doesn't exist
    await mkdir(cssDir, { recursive: true })
  } catch (error) {
    throw error
  }
}

// Run setup
await setup()