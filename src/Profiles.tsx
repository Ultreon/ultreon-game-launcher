import { invoke } from "@tauri-apps/api";

export class Profile {
  game!: string;
  version!: string;
  name!: string;
}

export const PROFILES: Array<Profile> = [];

invoke("load_profiles", { profiles: PROFILES })
