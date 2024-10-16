import type { Options } from 'tsup'

export const tsup: Options = {
  splitting: true,
  clean: true, // clean up the dist folder
  dts: true, // generate dts files
  format: ['cjs', 'esm'], // generate cjs and esm files
  minify: true,
  bundle: true,
  sourcemap: true,
  skipNodeModulesBundle: true,
  entryPoints: ['lib/index.ts'],
  target: 'es2020',
  outDir: 'dist',
  entry: ['lib/**/*.ts'], //include all files under lib
  tsconfig: 'tsconfig.release.json',
}
