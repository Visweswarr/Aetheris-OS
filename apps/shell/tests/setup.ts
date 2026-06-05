/**
 * @file setup.ts
 * @brief Test setup and global configuration
 */

// Global test setup
beforeEach(() => {
  // Clear all timers before each test
  jest.clearAllTimers();
});

afterEach(() => {
  // Clean up after each test
  jest.clearAllMocks();
});

// Mock console methods to reduce noise in tests
global.console = {
  ...console,
  // Uncomment to suppress console.log in tests
  // log: jest.fn(),
  // debug: jest.fn(),
  // info: jest.fn(),
  warn: jest.fn(),
  error: jest.fn(),
};

// Increase timeout for integration tests
jest.setTimeout(10000);
