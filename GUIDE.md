# 🧙‍♂️ The Murf Beginner's Guide: How to Make Windows Look Cool

Welcome, friend. 

If you are reading this, it means you’ve probably seen those gorgeous, perfectly color-coordinated Linux desktops on the internet and thought, *"I want that on my Windows machine, but I don't want to spend 40 hours configuring it."*

Good news: **You don't have to.** 

Murf is a magical little tray app that watches your wallpaper, extracts a Material You color palette using Matugen, and injects those colors into your terminal, status bar, and window manager. When you change your wallpaper, everything morphs to match.

No Rust. No dependencies. No 40-hour setup. Just download, run, and watch your desktop become a chameleon.

Let’s get you set up.

---

## 🚀 Step 1: The "It Just Works" Installation

Because we are nice people, Murf is a **single, standalone executable**. You don't need to install Python, Node, Rust, or Matugen. It's all baked into the `.exe`.

1. Go to the **Releases** page and download `Murf-v0.2.2-win64.zip`.
2. Extract `murf.exe` to a folder you won't accidentally delete (like `C:\Users\YourName\Tools\`).
3. Double-click `murf.exe`. 
4. Look down at your system tray (next to your clock). You'll see a Murf icon.
5. Right-click that icon. Click **Start at login**. Now you never have to think about it again.

---

## 🧠 Step 2: Understanding the Brain (`murf.toml`)

Murf is smart, but it lets you boss it around. It generates a configuration file at:
`%APPDATA%\Murf\murf.toml`

You don't need to change anything to get started, but this is where you can:
*   **Change the Morph Speed:** Look for `settle_delay_ms = 500`. That means it waits half a second after you change your wallpaper before applying the theme. Drop it to `200` for instant snapping, or raise it to `1500` if your PC is slow.
*   **Turn apps on/off:** If you don't use Cava, set `[apps.cava] enabled = false`.

---

## 🎨 Step 3: Setup Your Status Bar (YASB)

YASB is amazing, but CSS (the language used to style it) is a bit of a diva. 

Since v0.2.2, Murf automatically injects a theme block into your `styles.css`. But how it works depends entirely on how you wrote your CSS. **Pick the scenario that matches your setup:**

### Scenario A: You use Variables (The "Smart" Way)
If your `styles.css` has variables like `var(--background)` or `var(--text)` (this is how most modern themes are written), **you don't have to do anything**. 

Murf will inject the new colors at the bottom of your file, and YASB will morph instantly. Just sit back and enjoy.

### Scenario B: You use Hardcoded Colors (The "Dumb" Way)
If your `styles.css` has literal hex codes slapped onto widgets (like `.widget { background: #1e1e2e; }`), **Murf cannot override them automatically.** CSS doesn't work that way; hardcoded values ignore variables. 

You have two choices to fix this:
1. **The Easy Way:** Open the `examples/yasb-styles.css` file from the Murf repo. Copy the `:root` block and paste it over your existing colors. Then, do a "Find and Replace" in your editor, replacing `#1e1e2e` with `var(--background)`. It takes 5 minutes.
2. **The Clean Way:** Swap to a modern YASB theme that uses variables. (Murf now works perfectly with them).

*(Note: If you ever want to undo Murf's changes, just open your terminal and run `murf.exe --unapply`. It deletes the Murf block and restores your original file).*

---

## 🪟 Step 4: Setup Your Window Borders (GlazeWM)

GlazeWM is written in YAML. YAML is like that one strict teacher who will fail you if you use a Tab instead of 4 Spaces.

**Good news:** In v0.2.2, GlazeWM is completely zero-config. We dropped the complicated YAML anchors. Murf now just finds your window borders and replaces the colors directly.

Just open your GlazeWM `config.yaml` and make sure your `window_effects` section has the standard `border` -> `color` lines:

```yaml
window_effects:
  focused_window:
    border:
      enabled: true
      color: '#7AA2F7'    # <-- Murf will automatically replace this
  other_windows:
    border:
      enabled: true
      color: '#3B4261'    # <-- And this
```

That's it. Don't touch anything else. Murf will scan the file, find the focused and unfocused borders, and morph them with your wallpaper.

---

## 🐚 Step 5: Setup Your Shell (PowerShell Profile)

If you use **Starship** and **Fastfetch**, they need to know where Murf is writing the themes.

1. Open your PowerShell profile by typing this in your terminal: `notepad $PROFILE`
2. Paste the following block into the file (you can replace your existing Fastfetch/Starship commands with this):

```powershell
# Run Fastfetch using Murf's dynamically generated colors
if (Get-Command fastfetch -ErrorAction SilentlyContinue) {
    fastfetch
}

# Initialize Starship prompt
if (Get-Command starship -ErrorAction SilentlyContinue) {
    Invoke-Expression (&starship init powershell)
}
```

3. Save and close. Open a new terminal window.

*(Notice how you don't need `--config` anymore? Murf v0.2.2 directly injects colors into your existing `config.jsonc`, so the standard `fastfetch` command just works).*

---

## 🕹️ Step 6: Daily Driving Murf (The CLI)

You really don't have to do anything. But if you want to feel like a hacker, Murf comes with a few CLI commands:

*   **Force a color change:** Press `Ctrl + Alt + M` anywhere on your desktop, or right-click the tray and hit "Apply now".
*   **Wipe the theme:** `murf.exe --unapply` -> Removes the Murf block from YASB instantly.
*   **What is it doing?:** `murf.exe --dry-run` -> Tells you if it sees your MURF block and what it *would* inject. Useful for debugging.
*   **Pause watching:** Right-click the tray icon and check "Pause watching". Useful if you are doing a presentation.

---

## 🚨 Troubleshooting (When things go sideways)

**"My CLI apps (Cava, Chronoterm) aren't changing colors!"**
Unlike GUI apps, terminal apps can't safely be restarted in the background without breaking your layout. Murf updates their config files instantly, but they won't show the new colors until you close them and open them again.

**"My YASB colors aren't changing!"**
You probably have hardcoded hex codes in your CSS. Go back to **Step 3, Scenario B** and convert your colors to variables (`var(--background)`).

**"Fastfetch is throwing a 'Logo padding values must be positive integers' error!"**
Your `config.jsonc` got corrupted by an older version of Murf. Open `~/.config/fastfetch/config.jsonc` and ensure your padding uses clean numbers (like `1`), not decimals (like `1.0`).

**"GlazeWM says it can't find something / won't reload."**
You probably have a YAML syntax error (a missing space, or a stray character). Murf does targeted text replacement, so if it fails, it's usually a formatting issue in the rest of your config.

---

### 🎉 You're Done!

You now have a desktop that morphs with your wallpaper. Go change your background to a dark anime scene, watch your terminal turn purple, and take a screenshot for Reddit. 

If you find a bug, open an issue on GitHub. If you love it, leave a star. Happy ricing! 🚀