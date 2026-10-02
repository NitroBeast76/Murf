// Wallpaper watcher: registry notify + polling fallback, with debounce.
// Emits on a channel when the wallpaper has settled.

// TODO: RegNotifyChangeKeyValue on HKCU\Control Panel\Desktop\Wallpaper
// TODO: poll SPI_GETDESKWALLPAPER every 3s
// TODO: reset timer to now + settle_delay_secs on each signal
// TODO: on fire, send wallpaper path over mpsc::Sender<String>
