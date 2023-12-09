import { Binary } from 'binary-install'
import os from 'os'

async function getBinary() {
  const platform = getPlatform()
  const pkg_info = await import('../package.json')
  const { name, version } = pkg_info
  const url = `https://github.com/orbit-solutions-llc//${name}/releases/download/v${version}/${name}-${platform}.tar.gz`
  return new Binary(url, { name });
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
