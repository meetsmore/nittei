/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  bail: true,
  testMatch: ['**/*.spec*'],
  watchman: false,
  transform: {
    // Transform TypeScript source files with a CJS-targeted tsconfig
    '^.+\\.tsx?$': [
      'ts-jest',
      {
        tsconfig: '<rootDir>/tsconfig.jest.json',
      },
    ],
    // Transform ESM-only node_modules (ky ships ESM-only) to CJS for Jest
    '^.+\\.js$': [
      'ts-jest',
      {
        tsconfig: '<rootDir>/tsconfig.jest.json',
        diagnostics: false,
      },
    ],
  },
  // In pnpm, all packages live under:
  //   node_modules/.pnpm/<pkg>@<ver>/node_modules/<pkg>/
  // Jest follows symlinks so we must match the real .pnpm path.
  //
  // This pattern ignores (= does NOT transform) everything in the pnpm store
  // EXCEPT ky (which ships ESM-only and must be transformed to CJS).
  //
  // We intentionally do NOT include a separate "node_modules/(?!…)" catch-all
  // because that would match the inner "/node_modules/ky/" segment of pnpm paths
  // and accidentally ignore ky again.
  transformIgnorePatterns: [
    'node_modules/\\.pnpm/.+/node_modules/(?!(ky)/)',
  ],
}
