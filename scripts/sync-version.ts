import fs from "fs";
import path from "path";

const packageJsonPath = path.resolve("package.json");
const tauriConfPath = path.resolve("src-tauri/tauri.conf.json");
const cargoTomlPath = path.resolve("src-tauri/Cargo.toml");

function syncVersion() {
  try {
    // Read version from package.json
    const pkg = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8"));
    const version = pkg.version;
    console.log(`Syncing version ${version} from package.json...`);

    // Update tauri.conf.json
    const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, "utf-8"));
    tauriConf.version = version;
    fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");
    console.log(`Updated src-tauri/tauri.conf.json to ${version}`);

    // Update Cargo.toml
    let cargoToml = fs.readFileSync(cargoTomlPath, "utf-8");
    // Matches version = "x.y.z" at the start of a line (top-level package version)
    const cargoVersionRegex = /^version = "([^"]+)"/m;
    const match = cargoToml.match(cargoVersionRegex);

    if (match) {
      cargoToml = cargoToml.replace(cargoVersionRegex, `version = "${version}"`);
      fs.writeFileSync(cargoTomlPath, cargoToml);
      console.log(`Updated src-tauri/Cargo.toml to ${version}`);
    } else {
      console.warn("Could not find version key in src-tauri/Cargo.toml");
    }
  } catch (error) {
    console.error("Error syncing versions:", error);
    process.exit(1);
  }
}

syncVersion();
