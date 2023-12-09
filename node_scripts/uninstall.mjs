import { getBinary } from "./get_binary.mjs"

try {
  (await getBinary()).uninstall()
} catch (err) {
  console.log('Nothing to uninstall')
}
