import { Binary } from "@orb/simple-binary-install"
import pkg_info from '../package.json' with { type: 'json' }
import os from 'node:os'
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

const __dirname = dirname(fileURLToPath(import.meta.url))

function getBinary() {
  const platform = getPlatform()
  const { name, version } = pkg_info
  const url = `https://github.com/orbit-solutions-llc/${name}/releases/download/v${version}/${name}-${platform}.tar.gz`
  return new Binary(name, url, { installDirectory: join(__dirname, "../bin") });
}

function getPlatform() {
  const type = os.type();
  const arch = os.arch();

  if (type === 'Windows_NT' && arch === 'x64') { return 'win64' }
  if (type === 'Windows_NT') { return 'win32' }
  if (type === 'Linux' && arch === 'x64') { return 'linux' }
  if (type === 'Darwin' && arch === 'x64') { return 'macos-x86_64' }
  if (type === 'Darwin' && arch === 'arm64') { return 'macos-aarch64' }

  throw new Error(`Unsupported platform: ${type} ${arch}`)
}

export { getBinary }
