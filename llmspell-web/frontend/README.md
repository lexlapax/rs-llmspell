# llmspell-web Frontend

The web interface for the `rs-llmspell` platform. Built with React, TypeScript, Vite, and TailwindCSS.

## Development Mode

The frontend automatically detects development mode via Vite's `import.meta.env.MODE`:

- **Development** (`npm run dev`): `MODE = 'development'`
  - Authentication bypassed (matches backend dev_mode=true)
  - Dev mode banner visible
  - No token required to access protected routes

- **Production** (`npm run build`): `MODE = 'production'`
  - Full authentication required
  - Users must log in with API key
  - No dev mode banner

To test production mode locally:
```bash
npm run build
npm run preview  # Serves production build
```

To disable backend dev mode:
```bash
LLMSPELL_WEB_DEV_MODE=false ./target/debug/llmspell web start
```

## Setup

```bash
npm install
npm run dev
```

## Scripts

- `npm run dev`: Start development server
- `npm run build`: Build for production
- `npm run preview`: Preview production build
- `npm run lint`: Run ESLint
