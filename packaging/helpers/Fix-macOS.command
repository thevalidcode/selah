#!/bin/bash
set -e
APP="/Applications/Selah.app"
if [ ! -d "$APP" ]; then
  echo "Selah is not in /Applications yet. Drag it there first."
  read -n 1 -s -r -p "Press any key to close..."; exit 1
fi
xattr -dr com.apple.quarantine "$APP" || true
sudo xattr -cr "$APP" 2>/dev/null || true
echo "Done."
read -n 1 -s -r -p "Press any key to close..."