// Jest setup file for AI Bridge tests

// Mock console methods to reduce noise in tests
global.console = {
  ...console,
  // Uncomment to ignore console.log in tests
  // log: jest.fn(),
  // debug: jest.fn(),
  // info: jest.fn(),
  warn: jest.fn(),
  error: jest.fn(),
};

// Mock timers for testing
jest.useFakeTimers();

// Global test utilities
global.testUtils = {
  // Create mock abort signal
  createMockAbortSignal: (aborted = false) => ({
    aborted,
    reason: aborted ? new Error('Aborted') : undefined,
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
  }),

  // Create mock CapToken
  createMockCapToken: (overrides = {}) => ({
    tokenId: 'test-token-id',
    token: 'test-token-value',
    scopes: ['ai:chat', 'ai:tool.open_file'],
    expiresAt: Date.now() + 3600000,
    issuer: 'test-issuer',
    subject: 'test-subject',
    audience: 'ai-core-service',
    createdAt: Date.now(),
    ...overrides,
  }),

  // Wait for async operations
  waitFor: (ms) => new Promise(resolve => setTimeout(resolve, ms)),

  // Create mock event listener
  createMockEventListener: () => jest.fn(),
};

// Clean up after each test
afterEach(() => {
  jest.clearAllMocks();
  jest.clearAllTimers();
});

// Global test timeout
jest.setTimeout(10000);
