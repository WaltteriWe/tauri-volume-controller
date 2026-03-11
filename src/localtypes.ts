export type MediaInfo = {
    title: string;
    artist: string;
    tab_id: number;
    is_playing: boolean;
};

export type AppState = {
    volume: number;
    is_playing: boolean;
    current_media: MediaInfo | null;
};