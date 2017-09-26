#!/bin/bash
echo "Checking for FFMpeg..."

if ! ffmpeg_loc="$(type -p "ffmpeg")" || [ -z "$ffmpeg_loc" ]; then
	echo "FFMpeg not found! Please install it first!"
	exit 1
fi

echo "Creating default directorys..."
mkdir -p /ect/slms/renderer
mkdir -p /var/lib/slms/thumbnails

echo "Copy default configurations..."
cp -r images/icon.png /var/lib/slms/
cp -r configuration/* /etc/slms/

echo "Go into root directory.."
cd ../

echo "Building release..."
cargo build --release

echo "Installing..."
cp -r target/release/slms /usr/local/bin/

echo "Installation done. Make sure to change the Servers Configuration at /etc/slms/server.cfg"
