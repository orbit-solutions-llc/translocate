import { getBinary } from "./get_binary.js"

try {
  getBinary().uninstall()
} catch (_err) {
  console.log('Nothing to uninstall')
}
