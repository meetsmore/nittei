import type { Options } from 'tsdown'

export const tsdown: Options = {
  splitting: true,
  clean: true, // clean up the dist folder
  dts: true, // generate dts files
  format: ['cjs', 'esm'], // generate cjs and esm files
  minify: true,
  sourcemap: true,
  skipNodeModulesBundle: true,
  entry: ['lib/index.ts'],
  target: 'es2020',
  outDir: 'dist',
  entry: ['lib/**/*.ts'], //include all files under lib
  tsconfig: 'tsconfig.release.json',
}
