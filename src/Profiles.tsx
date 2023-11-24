import { invoke } from "@tauri-apps/api";

export class Profile {
  game!: string;
  version!: string;
  name!: string;
}

// eslint-disable-next-line react-refresh/only-export-components
export let PROFILES: Array<Profile> = [];

export async function load() {
  try {
    PROFILES = []
    PROFILES.push(...(await invoke("load_profiles") as Array<Profile>));
    console.log(PROFILES)
  } catch(error) {
    console.error(error)
  }
}

console.log(PROFILES)
