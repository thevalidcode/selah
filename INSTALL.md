# Installing Selah

Pick your platform below. If you see a security warning when opening
Selah, follow the "helper" step — it removes the warning permanently.

---

## macOS

1. Download the right `.dmg`:
   - **Apple Silicon (M1/M2/M3):** `Selah-x.y.z-aarch64.dmg`
   - **Intel:** `Selah-x.y.z-x64.dmg`

2. Open the `.dmg`, drag **Selah.app** into **Applications**.

3. Download **`Fix-macOS.command`** from this release.

4. **Right-click** `Fix-macOS.command` → **Open** → **Open** again.
   (You only need to do this once — macOS blocks new scripts by default.)

5. Terminal opens. It removes the block from Selah automatically.
   Press any key to close.

6. Open **Selah** from Applications. Done.

> If macOS still says "Selah is damaged," open Terminal and run:
> `sudo xattr -cr /Applications/Selah.app`

---

## Windows

1. Download **`Selah-Setup-x.y.z.exe`**.

2. Run it. If SmartScreen appears, click **More info → Run anyway**.

3. Install Selah normally.

4. Download **`Fix-Windows.bat`** from this release.

5. **Right-click** `Fix-Windows.bat` → **Run**. It removes the
   "downloaded from internet" flag from Selah's files.

6. Launch Selah from the Start menu. Done.

---

## Linux

1. **AppImage:** download `Selah-x.y.z.AppImage`, then:
   ```bash
   chmod +x Selah-*.AppImage
   ./Selah-*.AppImage
   ```
