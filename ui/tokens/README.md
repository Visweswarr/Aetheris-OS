# Aetheris OS Design Tokens

This directory contains generated design tokens for Aetheris OS.

## Files

- `tokens.css` - CSS custom properties
- `tokens.json` - JSON format
- `tokens.rs` - Rust constants
- `tokens.ts` - TypeScript definitions and utilities
- `index.ts` - TypeScript exports

## Usage

### CSS
```css
@import './tokens.css';

.my-component {
  color: var(--primary-500);
  padding: var(--spacing-4);
}
```

### TypeScript
```typescript
import { tokens, getToken, getCSSVar } from './tokens';

const primaryColor = getToken('primary-500');
const cssVar = getCSSVar('spacing-4');
```

### Rust
```rust
use aetheris_tokens::tokens::*;

let primary_color = PRIMARY_500;
```

## Source

Generated from: `design/Design-Tokens.yaml`
Version: 1.0.0
Last Modified: 2024-01-01T00:00:00Z
Generated: 2025-09-07T17:01:53.024Z
