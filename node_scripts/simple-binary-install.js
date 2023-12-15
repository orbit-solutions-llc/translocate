import axios from "axios";
import tar from "tar";
import { existsSync, lstatSync, mkdirSync, readdirSync, rmdirSync, unlinkSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, dirname } from "node:path";
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));


/**
 * `rm -rf` for node.js copied from the [rmrf](https://www.npmjs.com/package/rmrf) package
 * 
 * Copyright (c) 2013 Darcy Murphy
 * @param {string} dirPath 
 */
const rimraf = function(dirPath) {
  if (existsSync(dirPath)) {
    let files = readdirSync(dirPath)
    if (files && files.length > 0) {
      for (let i = 0; i < files.length; i++) {
        let filePath = dirPath + '/' + files[i]
        if (lstatSync(filePath).isDirectory())
          rimraf(filePath)
        else
          unlinkSync(filePath)
      }
    }
    rmdirSync(dirPath)
  }
};

/**
 * Prints an error message and then exits program with an error signal.
 * @param {string} msg - message to be printed to the console
 */
const error = msg => {
  console.error(msg);
  process.exit(1);
};

/**
 * "Simplified" version of binaray-install package, modified to use ESM.
 * 
 * https://www.npmjs.com/package/binary-install
 */
class Binary {
  /**
   * Constructor for the Binary class.
   * @param {string} name 
   * @param {string} url 
   * @param {{installDirectory: string}=} config 
   */
  constructor(name, url, config) {
    let errors = [];
    if (typeof url !== "string") {
      errors.push("url must be a string");
    } else {
      try {
        new URL(url);
      } catch (e) {
        errors.push(e);
      }
    }
    if (name && typeof name !== "string") {
      errors.push("name must be a string");
    }

    if (!name) {
      errors.push("You must specify the name of your binary");
    }

    if (
      config &&
      config.installDirectory &&
      typeof config.installDirectory !== "string"
    ) {
      errors.push("config.installDirectory must be a string");
    }

    if (errors.length > 0) {
      let errorMsg =
        "One or more of the parameters you passed to the Binary constructor are invalid:\n";
      errors.forEach(error => {
        errorMsg += error;
      });
      errorMsg +=
        '\n\nCorrect usage: new Binary("my-binary", "https://example.com/binary/download.tar.gz", "v1.0.0")';
      error(errorMsg);
    }
    this.url = url;
    this.name = name;
    this.installDirectory =
      config?.installDirectory || join(__dirname, "node_modules", ".bin");

    if (!existsSync(this.installDirectory)) {
      mkdirSync(this.installDirectory, { recursive: true });
    }

    this.binaryPath = join(
      this.installDirectory,
      `${this.name}`
    );
  }

  exists() {
    return existsSync(this.binaryPath);
  }

  install(fetchOptions, suppressLogs = false) {
    if (this.exists()) {
      if (!suppressLogs) {
        console.error(
          `${this.name} is already installed, skipping installation.`
        );
      }
      return Promise.resolve();
    }

    if (existsSync(this.installDirectory)) {
      rimraf(this.installDirectory);
    }

    mkdirSync(this.installDirectory, { recursive: true });

    if (!suppressLogs) {
      console.error(`Downloading release from ${this.url}`);
    }

    return axios({ ...fetchOptions, url: this.url, responseType: "stream" })
      .then(res => {
        return new Promise((resolve, reject) => {
          const sink = res.data.pipe(
            tar.x({ strip: 1, C: this.installDirectory })
          );
          sink.on("finish", () => resolve());
          sink.on("error", err => reject(err));
        });
      })
      .then(() => {
        if (!suppressLogs) {
          console.error(`${this.name} has been installed!`);
        }
      })
      .catch(e => {
        error(`Error fetching release: ${e.message}`);
      });
  }

  run(fetchOptions) {
    const promise = !this.exists()
      ? this.install(fetchOptions, true)
      : Promise.resolve();

    promise
      .then(() => {
        const [, , ...args] = process.argv;

        const options = { cwd: process.cwd(), stdio: "inherit" };

        const result = spawnSync(this.binaryPath, args, options);

        if (result.error) {
          error(result.error);
        }

        process.exit(result.status);
      })
      .catch(e => {
        error(e.message);
        process.exit(1);
      });
  }
}

export { Binary };