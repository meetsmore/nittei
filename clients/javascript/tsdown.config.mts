import { defineConfig } from 'tsdown'

export default defineConfig({
  clean: true, // clean up the dist folder
  dts: true, // generate dts files
  format: ['cjs', 'esm'], // generate cjs and esm files
  minify: true,
  sourcemap: true,
  entry: ['lib/index.ts'],
  target: 'es2020',
  outDir: 'dist',
  tsconfig: 'tsconfig.release.json',
})
