# Assistant UI Tests

This directory contains comprehensive tests for the Aetheris Assistant UI application.

## Test Structure

### `assistant-ui.spec.ts`
Core UI functionality tests including:
- Main interface display
- Message input and sending
- Chat interface behavior
- Settings panel
- Keyboard shortcuts
- Toolbar actions

### `electron.spec.ts`
Electron-specific tests including:
- Window creation and properties
- Hotkey registration
- IPC communication

### `accessibility.spec.ts`
Accessibility compliance tests including:
- Heading structure
- Form labels and ARIA attributes
- Keyboard navigation
- Screen reader support
- Focus management

### `performance.spec.ts`
Performance and efficiency tests including:
- Load time benchmarks
- Large message handling
- Memory usage
- Concurrent operations
- Streaming performance

### `security.spec.ts`
Security and safety tests including:
- Content Security Policy (CSP)
- XSS prevention
- Input sanitization
- File access security
- Network request control

### `integration.spec.ts`
End-to-end integration tests including:
- Complete chat flows
- Settings persistence
- Error recovery
- Window state management
- Theme switching

## Running Tests

### Prerequisites
- Node.js 18+
- npm or yarn
- Playwright installed

### Installation
```bash
npm install
npx playwright install
```

### Run All Tests
```bash
npm test
```

### Run Specific Test Files
```bash
npx playwright test assistant-ui.spec.ts
npx playwright test accessibility.spec.ts
npx playwright test performance.spec.ts
npx playwright test security.spec.ts
npx playwright test integration.spec.ts
```

### Run Tests in Headed Mode
```bash
npx playwright test --headed
```

### Run Tests in Debug Mode
```bash
npx playwright test --debug
```

## Test Configuration

Tests are configured in `playwright.config.ts` with:
- Multiple browsers (Chromium, Firefox, WebKit)
- Different viewport sizes
- Test timeouts and retries
- Screenshot and video capture on failure

## Mocking

Tests use mocked Electron APIs to simulate:
- AI Core Service communication
- Clipboard operations
- File system access
- Network requests

## Continuous Integration

Tests are designed to run in CI environments with:
- Headless browser execution
- Parallel test execution
- Artifact collection on failure
- Performance benchmarks

## Contributing

When adding new tests:
1. Follow the existing test structure
2. Use descriptive test names
3. Include proper setup and teardown
4. Mock external dependencies
5. Test both success and error cases
6. Include accessibility and security considerations
