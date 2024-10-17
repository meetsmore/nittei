import fs from 'node:fs/promises'

export async function generateIndexFiles(path: string) {
  const files = await fs.readdir(path, { withFileTypes: true })

  // Process folders
  const folders = files.filter(file => file.isDirectory())
  await Promise.all(
    folders.map(folder => generateIndexFiles(`${path}/${folder.name}`))
  )

  // Process files
  const notIndexFiles = files
    .filter(file => file.isFile())
    .filter(file => file.name !== 'index.ts')
    .map(file => file.name.replace('.ts', ''))

  const indexContent = notIndexFiles
    .map(file => {
      return `export * from './${file}'`
    })
    .join('\n')

  await fs.writeFile(`${path}/index.ts`, indexContent)
}

void generateIndexFiles('./lib/gen_types')
