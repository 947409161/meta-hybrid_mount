MODDIR="${0%/*}"
BASE_DIR="/data/adb/hybrid-mount"

mkdir -p "$BASE_DIR"

BINARY="$MODDIR/hybrid-mount"
if [ ! -f "$BINARY" ]; then
  echo "ERROR: Binary not found at $BINARY"
  exit 1
fi

chmod 755 "$BINARY"
"$BINARY" 2>&1
EXIT_CODE=$?

if [ "$EXIT_CODE" = "0" ]; then
  /data/adb/ksud kernel notify-module-mounted
fi
exit $EXIT_CODE
