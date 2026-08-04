#!/bin/bash
set -ex
ELF=$1
STORAGE_CONSTANTS="src/storage/constants.rs"
IMAGE="target/littlefs.bin"

if [ -f "$STORAGE_CONSTANTS" ]; then
    OFFSET=$(grep "PARTITION_OFFSET" "$STORAGE_CONSTANTS" | head -n 1 | cut -d'=' -f2 | tr -d ' ;')
else
    OFFSET="0x300000"
fi

if [ -f "$IMAGE" ]; then
    echo "Flashing $ELF and $IMAGE at $OFFSET"
    espflash write-bin "$OFFSET" "$IMAGE"
    espflash flash --chip esp32s3 --monitor --partition-table partition.csv "$ELF"
else
    echo "Warning: $IMAGE not found, flashing $ELF only"
    espflash flash --monitor --chip esp32s3 "$ELF"
fi
