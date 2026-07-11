#!/bin/sh

set -e

chmod +x app.sh

if [ -f /lib/ld-linux-armhf.so.3 ]; then
    PLAT=kindlehf
else
    PLAT=kindlepw2
fi

cat > /mnt/us/documents/com.bd452.emberdemo.sh << EOF
#!/bin/sh
exec /var/local/kmc/${PLAT}/bin/kpm launch com.bd452.emberdemo
EOF
chmod +x /mnt/us/documents/com.bd452.emberdemo.sh

echo "Ember Demo installed. Open com.bd452.emberdemo.sh from Documents to launch."
