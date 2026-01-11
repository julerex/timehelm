---
description: TypeScript coding conventions for the client
globs: 'client/**/*.ts'
alwaysApply: false
---

## TypeScript Conventions

- **Classes for major components** - `GameClient`, `Player`, `CameraController`, etc.
- **Readonly for immutable properties** - `private readonly user: User`
- **Null initialization with type annotation** - `private scene: THREE.Scene | null = null`
- **Section comments** - Use `// --- Section Name ---` to organize methods
- **Explicit exports** - Re-export types from index files when needed
- **No explicit return types required** (ESLint rule disabled)
- **Prefix unused params with underscore** - `(_unused) => {}`

Example structure:
```typescript
export class MyComponent {
    private readonly config: Config;
    private state: State | null = null;

    constructor(config: Config) {
        this.config = config;
    }

    // --- Public Methods ---
    public init(): void { }
    public dispose(): void { }

    // --- Private Methods ---
    private setupX(): void { }
    private handleY(): void { }
}
```

