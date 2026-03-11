import { invoke } from "@tauri-apps/api/core";
import { AppState, MediaInfo } from "./localtypes";



async function fetchState(): Promise<AppState> {
  return await invoke<AppState>("get_current_media")
}

async function setVolume(volume:number): Promise<void> {
  await invoke("set_volume", { volume })
}

async function togglePlayPause(): Promise<boolean> {
  return await invoke<boolean>("toggle_play_pause")
}

function renderState(state: AppState): void {
  const titleEl = document.querySelector<HTMLElement>("#track-title")!;
  const artistEl = document.querySelector<HTMLElement>("#track-artist")
  const playBtn = document.querySelector<HTMLElement>("#play-pause")
  const volumeSlider = document.querySelector<HTMLInputElement>("#volume-slider")
  const volumeDisplay = document.querySelector<HTMLElement>("#volume-value")



  if (state.current_media) {
    titleEl.textContent = state.current_media.title || "Unknown title";
    artistEl!.textContent = state.current_media.artist || "";
  } else {
    titleEl.textContent = "No media playing";
    artistEl!.textContent = "";
  }


  playBtn!.textContent = state.is_playing ? "Pause" : "Play";
}

