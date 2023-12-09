import { getBinary } from "./get_binary.js"

try {
  getBinary().uninstall()
} catch (err) {
  console.log('Nothing to uninstall')
}
