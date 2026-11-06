# 🧙 Murf: The Beginner's Guide to Wallpaper-Driven Windows Theming

<p align="center">
  <img src="assets/murf.png" alt="Murf" width="120">
</p>

<p align="center">
  <strong>Make Windows look cool. Without sacrificing your entire weekend.</strong>
</p>

<p align="center">
  <a href="../README.md">← Back to README</a> ·
  <a href="https://github.com/NitroBeast76/Murf/releases">Download Murf</a> ·
  <a href="https://github.com/NitroBeast76/Murf/issues">Report a Problem</a>
</p>

---

## 👋 Welcome, friend.

If you're reading this, you've probably seen those gorgeous, perfectly color-coordinated Linux desktops on the internet and thought:

*"I want that on my Windows machine. I just don't want to spend 40 hours configuring it."*

Good news.

**You don't have to.**

Murf is a magical little tray application that watches your wallpaper, extracts a Material You color palette using [Matugen](https://github.com/InioX/matugen), and injects those colors into your terminal, status bar, window manager, and Windows itself.

Change your wallpaper. Your desktop changes with it.

No manually editing twelve configuration files every time you feel like changing your wallpaper. No spending your evening hunting down hexadecimal colors. No existential crisis because your terminal is still purple while your wallpaper is orange.

Just download, run, and watch your desktop become a chameleon.

This guide will take you from a completely untouched installation to a fully dynamic Windows rice.

---

## 📋 Before We Begin

Murf is designed to work with popular Windows customization tools.

Here's what we'll be using:

| Application | Purpose | Required? |
|---|---|---|
| Murf | Generates and applies dynamic themes | Yes |
| YASB | Custom Windows status bar | Optional |
| GlazeWM | Tiling window manager and window effects | Optional |
| Starship | Custom shell prompt | Optional |
| Fastfetch | Terminal system information | Optional |
| Cava | Terminal audio visualizer | Optional |
| Chronoterm | Terminal customization | Optional |

You don't need to install everything.

Murf automatically skips applications that aren't installed or enabled.

**The only mandatory component is Murf itself.**

---

# 🚀 Step 1: The "It Just Works" Installation

Murf is distributed as a standalone executable.

You don't need to install Python, Node.js, Rust, or Matugen separately to use the packaged release.

### 1. Download Murf

Visit the [Releases page](https://github.com/NitroBeast76/Murf/releases) and download:

`Murf-v0.2.2-win64.zip`

Extract the archive.

### 2. Give Murf a permanent home

Place `murf.exe` somewhere you won't accidentally delete.

For example:

```text
C:\Users\YourName\Tools\Murf\
```

### 3. Launch Murf

Double-click `murf.exe`.

Look down at your system tray, next to the clock.

You'll find the Murf icon waiting there.

### 4. Enable startup

Right-click the Murf tray icon and select:

**Start at login**

Congratulations.

Murf is now running in the background and ready to monitor your wallpaper.

You may proceed to admire your desktop.

---

# 🧠 Step 2: Understanding the Brain — `murf.toml`

Murf is smart, but it still lets you boss it around.

On its first run, Murf generates a configuration file at:

```text
%APPDATA%\Murf\murf.toml
```

Paste that path into File Explorer's address bar to open the directory.

The configuration file is heavily commented and contains settings for Murf's behavior and individual application adapters.

You don't need to touch anything to get started.

However, there are a few settings worth knowing.

### Adjusting the morph speed

Look for:

```toml
[general]
settle_delay_ms = 500
```

This determines how long Murf waits after detecting a wallpaper change before applying the theme.

| Value | Meaning |
|---|---|
| `200` | Very aggressive. Almost instant. |
| `500` | Snappy and generally sensible. |
| `1000` | A little more relaxed. |
| `1500` | Useful if your wallpaper application is slow. |

For most systems, the default `500` is a good starting point.

### Turning applications on or off

Suppose you don't use Cava.

Find its adapter section and disable it:

```toml
[apps.cava]
enabled = false
```

The same principle applies to other supported applications.

You can keep your desktop's theming modular instead of forcing every application to participate.

---

# 🎨 Step 3: The Most Important Concept — CSS Variables

This is the section that separates people who merely *use* dynamic themes from people who actually understand how to customize them.

If you use YASB, understanding CSS variables will make your life significantly easier.

## What exactly is a CSS variable?

A CSS variable is a reusable value that you define once and reference throughout your stylesheet.

Instead of writing the same color everywhere, you give it a name.

For example:

```css
:root {
    --background: #1e1e2e;
    --text: #cdd6f4;
    --accent: #89b4fa;
}
```

These three variables represent your background, text, and accent colors.

Now, instead of hardcoding colors into every widget, you can reference them:

```css
.widget {
    background-color: var(--background);
    color: var(--text);
}

.widget:hover {
    background-color: var(--accent);
}
```

The result?

Every widget using `var(--background)` automatically receives the same background color.

If you change the variable, every widget using it changes too.

**One color definition. Hundreds of potential uses.**

That's the entire idea behind reusable CSS variables.

## How Murf uses CSS variables

Murf generates a dynamic color palette from your wallpaper.

For YASB, Murf injects its theme block into your stylesheet.

When your widgets use compatible CSS variables, they can automatically pick up the newly injected colors.

For example:

```css
:root {
    --background: #1e1e2e;
    --text: #cdd6f4;
    --accent: #89b4fa;
}
```

A widget using:

```css
.widget {
    background-color: var(--background);
    color: var(--text);
}
```

doesn't care whether the background is purple, orange, green, or pink.

It simply uses whatever value `--background` currently contains.

When Murf updates the variable, the widget's appearance changes accordingly.

### Why hardcoded colors are a problem

Consider this stylesheet:

```css
.widget {
    background-color: #1e1e2e;
    color: #cdd6f4;
}

.widget:hover {
    background-color: #89b4fa;
}
```

Those colors are hardcoded.

Murf can inject new variables into the stylesheet, but it cannot magically turn those literal hexadecimal values into variable references.

The widget never references the variables in the first place.

This is why some YASB themes morph beautifully while others stubbornly retain their original colors.

**Murf changes the palette. Your stylesheet determines whether your widgets actually use it.**

---

## Scenario A: Your stylesheet already uses variables

If your `styles.css` already contains something like:

```css
background-color: var(--background);
color: var(--text);
```

you're in excellent shape.

Since v0.2.2, Murf automatically injects its theme block into the stylesheet.

If your stylesheet references the corresponding variables, you generally don't need to do anything else.

Change your wallpaper.

Watch YASB morph.

Enjoy the fact that someone else did the tedious part.

---

## Scenario B: Your stylesheet uses hardcoded hexadecimal colors

This is where the fun begins.

Suppose your stylesheet contains:

```css
.widget {
    background-color: #1e1e2e;
    color: #cdd6f4;
}

.widget:hover {
    background-color: #89b4fa;
}
```

You want to convert it to a dynamic stylesheet.

There are two ways.

### Method 1: The Easy Way

Open:

```text
examples/yasb-styles.css
```

in the Murf repository.

Copy its `:root` block into your own `styles.css`.

Then replace your hardcoded colors with the appropriate variables.

For example:

```css
/* BEFORE */

.widget {
    background-color: #1e1e2e;
    color: #cdd6f4;
}
```

Becomes:

```css
/* AFTER */

.widget {
    background-color: var(--background);
    color: var(--text);
}
```

Repeat for the other colors.

That's it.

### Method 2: Convert your own stylesheet manually

This is the method I recommend if you want complete control over your theme.

You don't need to redesign your stylesheet.

You don't need to change widget layouts.

You don't even need to understand every CSS property.

You only need to replace the colors.

#### Step 1: Identify your existing colors

Open your YASB `styles.css`.

Look for hexadecimal values beginning with `#`.

For example:

```css
#1e1e2e
#181825
#313244
#cdd6f4
#89b4fa
#f38ba8
```

These are color literals.

Your editor's Find function can help locate them.

#### Step 2: Understand the Murf variables

Murf's injected theme block defines the dynamic color variables.

The exact variables available depend on the theme block generated by Murf.

A typical Material You-style palette might include roles such as:

| Variable | Intended role |
|---|---|
| `--background` | Main background |
| `--surface` | Secondary surfaces |
| `--surface-alt` | Alternate surface |
| `--text` | Main foreground text |
| `--text-muted` | Secondary text |
| `--primary` | Main accent |
| `--secondary` | Secondary accent |
| `--tertiary` | Additional accent |

**Important:** These names are illustrative. Before using a variable, check the actual `:root` block generated by Murf or the example stylesheet.

CSS variable names must match exactly.

`var(--primary)` and `var(--murf_primary)` are not interchangeable.

#### Step 3: Replace the colors

Let's take a real example.

Your original stylesheet:

```css
:root {
    --widget-bg: #1e1e2e;
    --widget-text: #cdd6f4;
    --widget-accent: #89b4fa;
}

.widget {
    background-color: #1e1e2e;
    color: #cdd6f4;
}

.widget:hover {
    background-color: #89b4fa;
}
```

You can convert it into:

```css
.widget {
    background-color: var(--background);
    color: var(--text);
}

.widget:hover {
    background-color: var(--primary);
}
```

Now the widget is using Murf's dynamic colors.

Notice that the original `:root` definitions are no longer needed for those three colors.

Murf provides the dynamic values instead.

#### Step 4: Use Find and Replace

If your stylesheet contains hundreds of repeated colors, doing this manually is tedious.

Use your editor's Find and Replace feature.

For example:

| Find | Replace |
|---|---|
| `#1e1e2e` | `var(--background)` |
| `#cdd6f4` | `var(--text)` |
| `#89b4fa` | `var(--primary)` |

Replace only colors that genuinely serve the same purpose.

If `#1e1e2e` is used for both a background and a border, you may want different variables for those roles.

**Do not blindly replace every hexadecimal color with the same variable.**

Your desktop will technically become dynamic, but the visual hierarchy may suffer.

---

## A Better Way: Think in Color Roles

When converting a stylesheet, don't think:

*"This hex code is blue. Which blue variable replaces it?"*

Think:

*"What job is this color doing?"*

A color's purpose matters more than its exact appearance.

For example:

```css
.widget {
    background-color: #1e1e2e;
    color: #cdd6f4;
    border: 1px solid #313244;
}
```

The three colors have different responsibilities.

| Element | Role |
|---|---|
| Widget background | Background |
| Widget text | Foreground |
| Widget border | Border / outline |

When converting to Murf variables, preserve those roles.

```css
.widget {
    background-color: var(--background);
    color: var(--text);
    border: 1px solid var(--outline);
}
```

Use the actual outline variable available in Murf's generated theme.

This is especially important when working with Material You palettes.

Material You isn't simply a collection of random colors.

It uses color roles to maintain contrast and visual relationships across an interface.

By using those roles instead of hardcoded colors, your stylesheet can adapt to very different wallpapers without losing its structure.

---

# 🪟 Step 4: Setup Your Window Borders — GlazeWM

GlazeWM is written in YAML.

YAML is like that one strict teacher who will fail you for using a Tab instead of four spaces.

Fortunately, Murf v0.2.2 makes this considerably easier.

## How Murf handles GlazeWM

Murf can find and replace the relevant border colors directly in your GlazeWM configuration.

You no longer need to manually maintain complicated YAML anchors for the supported border colors.

However, Murf still needs a recognizable border configuration to work with.

### 1. Open your GlazeWM configuration

Usually:

```text
~/.glzr/glazewm/config.yaml
```

Locate your `window_effects` section.

### 2. Make sure your borders are configured

For example:

```yaml
window_effects:
  focused_window:
    border:
      enabled: true
      color: '#7AA2F7'

  other_windows:
    border:
      enabled: true
      color: '#3B4261'
```

The hexadecimal colors are simply initial values.

Murf will automatically replace the supported border colors with its dynamic palette.

### 3. Save the configuration

That's it.

Murf can now morph your focused and unfocused window borders according to your wallpaper.

### A small but important warning

Murf performs targeted text replacement.

Keep your YAML valid.

Don't randomly rearrange the border structure or change indentation unless you understand what you're doing.

If GlazeWM stops reloading after an edit, check the YAML syntax first.

---

# 🐚 Step 5: Setup Your Shell — PowerShell, Starship & Fastfetch

Your terminal deserves attention too.

After all, it's where you'll spend most of your time pretending that a single `git commit` required three hours of work.

Murf supports dynamic theming for Starship and Fastfetch.

## PowerShell Profile

Your PowerShell profile is a script that runs automatically whenever you start a PowerShell session.

To open it, run:

```powershell
notepad $PROFILE
```

If PowerShell asks whether you want to create the file, select Yes.

Add the following:

```powershell
# Run Fastfetch if installed
if (Get-Command fastfetch -ErrorAction SilentlyContinue) {
    fastfetch
}

# Initialize Starship if installed
if (Get-Command starship -ErrorAction SilentlyContinue) {
    Invoke-Expression (&starship init powershell)
}
```

Save the file.

Open a new terminal window.

### Why don't we specify a Fastfetch config path?

Murf v0.2.2 injects its dynamic colors directly into your existing Fastfetch configuration.

The standard `fastfetch` command can therefore use the generated configuration without requiring an additional `--config` argument.

This also means your existing Fastfetch configuration remains important.

Keep a backup if you have heavily customized it.

### Custom Fastfetch ASCII art

Want something more interesting than the default Windows logo?

Murf supports a custom ASCII logo source.

In `murf.toml`:

```toml
[apps.fastfetch]
enabled = true
logo_source = "C:\\Users\\YourName\\my-ascii-art.txt"
```

Create the text file, add your ASCII art, and point Murf toward it.

You can have an Eiffel Tower, an anime character, a company logo, or something so incomprehensible that nobody will question your technical expertise.

---

# 🕹️ Step 6: Daily Driving Murf

Once everything is configured, you shouldn't need to interact with Murf very often.

But when you do, there are several useful options.

## Global hotkey

```text
Ctrl + Alt + M
```

Forces an immediate theme re-apply.

Useful when you've manually edited a configuration or simply want to refresh the current palette.

## System tray

Right-click Murf's tray icon.

| Action | What it does |
|---|---|
| Apply Now | Immediately re-applies the current theme |
| Pause Watching | Temporarily stops wallpaper monitoring |
| Toggle Adapters | Enable or disable individual applications |
| Start at login | Controls automatic startup |
| Quit | Exits Murf |

## CLI commands

Murf also provides command-line functionality.

### Remove Murf's theme

```powershell
murf.exe --unapply
```

This removes the supported Murf theme block and restores the original file where applicable.

### Dry run

```powershell
murf.exe --dry-run
```

Useful for inspecting what Murf detects and what it would inject without performing the actual apply operation.

This is particularly helpful when troubleshooting configuration issues.

**Tip:** Run CLI commands from the directory containing `murf.exe`, or use its full path.

---

# 🧪 Step 7: Experiment With Your Theme

Now that you understand how Murf works, here's where customization gets interesting.

You can start experimenting with:

- Different wallpapers.
- Different color palettes.
- Different YASB widget colors.
- More subtle or aggressive accent colors.
- Dark and light Windows themes.
- Custom Fastfetch logos.
- Different border colors for focused and unfocused windows.

## The golden rule

**Use variables for colors you want Murf to control.**

Keep hardcoded colors for things you deliberately want to remain unchanged.

For example, you might want your desktop's background to morph while keeping a specific warning indicator permanently red.

That is perfectly reasonable.

Dynamic theming is about control, not making every pixel change color.

---

# 🚨 Troubleshooting

Things occasionally go sideways.

That's normal.

Here's how to deal with the most common issues.

## "My YASB colors aren't changing!"

The most common reason is hardcoded CSS colors.

Check whether your widgets use:

```css
background-color: #1e1e2e;
```

instead of:

```css
background-color: var(--background);
```

If they're hardcoded, return to Step 3 and convert them.

Also verify that the variable actually exists in Murf's injected theme block.

## "Cava or Chronoterm isn't changing colors!"

That's expected.

Murf updates their configuration files immediately, but those applications cannot safely live-reload through Murf.

Close the application and launch it again.

The new colors should appear.

## "Fastfetch is throwing a Logo padding values error!"

An older Murf version may have left an invalid value in your Fastfetch configuration.

Open:

```text
~/.config/fastfetch/config.jsonc
```

Check your logo padding values.

Use positive integers where required.

For example:

```jsonc
"padding": {
    "top": 1,
    "left": 2
}
```

Avoid invalid values such as:

```jsonc
"padding": {
    "top": 1.0,
    "left": -2
}
```

If the problem persists, inspect the generated configuration and restore a known-good backup.

## "GlazeWM won't reload!"

Check your YAML.

Common problems include:

- Incorrect indentation.
- Tabs instead of spaces.
- Missing quotation marks.
- Invalid hexadecimal color syntax.
- A malformed `window_effects` section.

Murf's targeted replacements do not automatically correct unrelated YAML errors.

## "Murf isn't applying anything!"

Try the following:

1. Verify that Murf is running in the system tray.
2. Make sure wallpaper watching isn't paused.
3. Press `Ctrl + Alt + M`.
4. Verify that the relevant adapters are enabled.
5. Check your application configuration paths.
6. Inspect the Murf log.

The log is located at:

```text
%LOCALAPPDATA%\Murf\logs\murf.log
```

If you still can't figure out what's happening, open an issue on GitHub and include the relevant log entries.

---

# 🧹 Step 8: Uninstalling Murf

Leaving the dynamic theming lifestyle?

No hard feelings.

### 1. Quit Murf

Right-click the tray icon and select Quit.

### 2. Remove the executable

Delete `murf.exe` from its installation directory.

### 3. Remove configuration and logs

Optional:

```text
%APPDATA%\Murf\
%LOCALAPPDATA%\Murf\
```

### 4. Restore your old application themes

Murf's generated theme blocks may remain in application configuration files.

If you want to restore your original appearance, remove the Murf-related blocks or restore your previous backups.

Your applications will continue functioning normally.

They simply won't receive dynamic palette updates anymore.

---

# 🎉 Congratulations. You're officially ricing.

You now understand how to:

- Install Murf.
- Configure its adapters.
- Use CSS variables.
- Convert hardcoded CSS colors into dynamic variables.
- Theme YASB.
- Theme GlazeWM borders.
- Configure Starship and Fastfetch.
- Use Murf's hotkey and CLI.
- Troubleshoot common issues.

Most importantly, you understand *why* dynamic theming works.

That means you're no longer limited to copying someone else's configuration.

You can build your own.

Change your wallpaper to a dark anime scene.

Watch your terminal turn purple.

Switch to a golden fantasy landscape.

Watch the desktop morph again.

Then take a screenshot for Reddit.

## Happy ricing! 🚀

<p align="center">
  <strong>Murf</strong>
  <br>
  <em>One wallpaper. One palette. An entire desktop.</em>
</p>

<p align="center">
  <a href="https://github.com/NitroBeast76/Murf">⭐ Star Murf on GitHub</a>
</p>
