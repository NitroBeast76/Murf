# 🧙‍♂️ The Murf Beginner's Guide: How to Make Windows Look Cool

Welcome, friend. 

If you are reading this, it means you’ve probably seen those gorgeous, perfectly color-coordinated Linux desktops (you know the ones on r/unixporn) and thought, *"I want that on my Windows machine, but I don't want to spend 40 hours configuring it."*

Good news: **You don't have to.** 

Murf is a magical little tray app that watches your wallpaper, extracts a Material You color palette using Matugen, and injects those colors into your terminal, status bar, and window manager. When you change your wallpaper, everything morphs to match.

No Rust. No dependencies. No 40-hour setup. Just download, run, and watch your desktop become a chameleon.

Let’s get you set up.

---

## 🚀 Step 1: The "It Just Works" Installation

Because we are nice people, Murf is a **single, standalone executable**. You don't need to install Python, Node, Rust, or Matugen. It's all baked into the `.exe`.

1. Go to the **Releases** page and download `Murf-v0.2.1-win64.zip`.
2. Extract `murf.exe` to a folder you won't accidentally delete (like `C:\Users\YourName\Tools\`).
3. Double-click `murf.exe`. 
4. Look down at your system tray (next to your clock). You'll see a Murf icon.

**Right-click that icon.** Click **Start at login**. Now you never have to think about it again.

*Note: On first run, Murf will silently extract the Matugen color engine to your `%LOCALAPPDATA%` folder. You won't see a console window flash. It's magic.*

---

## 🧠 Step 2: Understanding the Brain (`murf.toml`)

Murf is smart, but it lets you boss it around. It generates a configuration file at:
`%APPDATA%\Murf\murf.toml`

Open it in Notepad. You don't need to change anything to get started, but this is where you can:
*   Turn specific apps on or off (`enabled = true/false`).
*   Change the morph speed (make it snap instantly, or smoothly wait 2 seconds).
*   Tell Murf where to find your apps if you use portable folders instead of the standard ones.

**The Golden Rule:** If Murf is acting weird, check the logs at `%LOCALAPPDATA%\Murf\logs\murf.log`. It will literally tell you what went wrong.

---

## 🎨 Step 3: Setup Your Status Bar (YASB)

YASB is amazing, but CSS (the language used to style it) is a bit of a diva. 

Because Murf has to write dynamic colors, it uses something called **CSS Variables**. Depending on how you set up YASB previously, you have two paths. **Pick ONE of the following options.**

### Path A: Your YASB is "Hardcoded" (You have hex codes like `#1e1e2e` everywhere)
If you look at your `styles.css` and see literal colors slapped directly onto widgets, CSS variables won't override them. You have to use our full replacement file.

1. Open the `examples/yasb-styles.css` file from the Murf repo.
2. Copy the **entire contents** of that file.
3. Open your own YASB `styles.css`.
4. Delete everything in your file.
5. Paste the example file in.
6. Save. Restart YASB.

*Why?* The example file replaces all hardcoded hexes with variables. Now, when Murf updates the colors, YASB actually listens.

### Path B: Your YASB is "Variable-Based" (You already use `var(--background)`)
You are a genius. This is much easier.

1. Open your YASB `styles.css`.
2. Scroll to the top and find your `:root { ... }` block.
3. Add this block inside it:
```css
    /* Murf Dynamic Colors */
    --background:        var(--murf-bg);
    --background2:       var(--murf-surface-high);
    --accent:            var(--murf-primary);
    --text:              var(--murf-fg);
    --accentText:        var(--murf-on-primary);
    --hover:             var(--murf-surface-high);
    --mutedBG:           var(--murf-surface-dim);
    --border:            var(--murf-outline);
    --redFlash:          var(--murf-error);
    --subtext:           var(--murf-secondary);
```
4. Save. Restart YASB.

*(If you get stuck, just copy the `examples/yasb-styles.css` file. We won't judge.)*

---

## 🪟 Step 4: Setup Your Window Borders (GlazeWM)

GlazeWM is written in YAML. YAML is like that one strict teacher who will fail you if you use a Tab instead of 4 Spaces. Be careful.

Murf uses **YAML Anchors** to theme GlazeWM. Think of an anchor as a label. Murf puts a label at the top of your config, and you just point your border colors to that label.

1. Open your GlazeWM `config.yaml`.
2. Find the `window_effects:` section.
3. Change your `color:` lines to look exactly like this:

```yaml
window_effects:
  focused_window:
    border:
      enabled: true
      color: *murf_primary    # <-- Use the Murf anchor here
    hide_title_bar:
      enabled: false
    corner_style:
      enabled: true
      style: 'rounded'
    transparency:
      enabled: true
      opacity: '98%'

  other_windows:
    border:
      enabled: true
      color: *murf_outline    # <-- And here
    # ... rest of your config
```

4. Save the file.
5. Reload GlazeWM (`Alt + Shift + R`).

*Note: Don't worry about defining `*murf_primary` anywhere. Murf automatically injects the definitions at the very top of your YAML file on every apply.*

---

## 🐚 Step 5: Setup Your Shell (PowerShell Profile)

If you use **Starship** and **Fastfetch**, they need to know where Murf is writing the themes.

1. Open your PowerShell profile by typing this in your terminal: `notepad $PROFILE`
2. Paste the following block into the file (you can replace your existing Fastfetch/Starship commands with this):

```powershell
# Run Fastfetch using Murf's dynamically generated config
if (Get-Command fastfetch -ErrorAction SilentlyContinue) {
    $murfConfig = "$HOME\.config\fastfetch\murf.jsonc"
    if (Test-Path $murfConfig) {
        fastfetch --config $murfConfig
    } else {
        fastfetch
    }
}

# Initialize Starship prompt
if (Get-Command starship -ErrorAction SilentlyContinue) {
    Invoke-Expression (&starship init powershell)
}
```

3. Save and close. Open a new terminal window.

**Want your own custom ASCII art in Fastfetch?**
By default, Fastfetch shows the Windows logo. If you want your own text art:
1. Save your ASCII art as a `.txt` file.
2. Open `%APPDATA%\Murf\murf.toml`.
3. Add this line under the `[apps.fastfetch]` section:
```toml
logo_source = "C:\\Users\\YourName\\my-ascii-art.txt"
```
*(Make sure to use double backslashes `\\` in the TOML file).*

---

## 🕹️ Step 6: Daily Driving Murf

You really don't have to do anything. But if you want to feel like a hacker:

*   **Force a color change:** Press `Ctrl + Alt + M` anywhere on your desktop. This forces Murf to re-read the wallpaper and apply the theme instantly.
*   **Pause watching:** Right-click the tray icon and check "Pause watching". Useful if you are doing a presentation and don't want your screen to flash colors.
*   **Toggle apps:** If you suddenly hate Cava, right-click -> Adapters -> uncheck Cava. Murf will stop bothering it.

---

## 🚨 Troubleshooting (When things go sideways)

**"My CLI apps (Cava, Chronoterm) aren't changing colors!"**
Unlike GUI apps, terminal apps can't safely be restarted in the background without breaking your terminal layout. Murf updates their config files instantly, but they won't show the new colors until you close them and open them again.

**"My YASB colors aren't changing!"**
You probably have hardcoded hex codes in your CSS. Go back to **Step 3** and use our full replacement example.

**"Murf won't start / The tray icon is missing."**
Did you delete the `assets` folder? You shouldn't have—Murf is a standalone `.exe`. If it's crashing, open `%LOCALAPPDATA%\Murf\logs\murf.log` and see what it's crying about.

**"GlazeWM says it can't find the *murf_primary anchor."**
You probably have a YAML syntax error (a missing space, or a stray character). Murf injects the block perfectly, so if it fails, it's usually a formatting issue in the rest of your config.

---

### 🎉 You're Done!

You now have a desktop that morphs with your wallpaper. Go change your background to a dark anime scene, watch your terminal turn purple, and take a screenshot for Reddit. 

If you find a bug, open an issue on GitHub. If you love it, leave a star. Happy ricing! 🚀
