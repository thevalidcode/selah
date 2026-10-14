# Installing Selah

Pick your platform below.

---

## macOS

1. Download the right `.dmg` for your Mac:
   - **Apple Silicon (M1/M2/M3):** `Selah-x.y.z-aarch64.dmg`
   - **Intel:** `Selah-x.y.z-x64.dmg`

2. Open the `.dmg` and drag **Selah.app** into **Applications**.

3. macOS will block Selah the first time, because this release is not
   yet code-signed. To allow it, run one command in Terminal:
   - Press `Cmd + Space`, type `Terminal`, press Enter.
   - Paste this and press Enter:

> `sudo xattr -cr /Applications/Selah.app`

- Type your Mac password when asked. **Nothing will appear as you
  type — that is normal.** Press Enter again.

4. Open **Selah** from Applications. It will launch normally.

> **Why is this needed?**
> Apple requires paid code signing ($99/year) for apps distributed
> outside the App Store. Once we have that set up, this step will
> disappear. Until then, the Terminal command above is the only
> workaround.

> **"Selah is damaged and can't be opened"** — same fix. The command
> above removes the flag macOS added when you downloaded it.

---

## Windows

1. Download both files from this release:

- **`Selah-Setup-x.y.z.exe`** (the installer)
- **`Fix-Windows.bat`** (the helper)

2. Put them in the **same folder** (e.g. your Downloads folder).

3. **Right-click** `Fix-Windows.bat` → **Run**.

If a blue "Windows protected your PC" window appears, click
**More info** → **Run anyway**. This happens once.

4. The helper removes the "downloaded from the internet" flag from
   the installer and launches it. Follow the installer prompts.

5. Open **Selah** from your Start menu. Done.

> **Why is this needed?**
> Windows flags any installer downloaded from the internet. The
> helper tells Windows the installer is safe, so you only see the
> warning once instead of during both the download and the install.

---

## Linux

No helper or extra steps needed — Linux does not quarantine binaries.

**AppImage (any distro):**

```bash
chmod +x Selah-*.AppImage
./Selah-*.AppImage
```
