Due to the need for a display and a GPU the deployment for this is a bit different. You can't just use Cloud Run, not to mention Cloud Run entails a separate DB which is sort of overkill for this processor since we're only tracking what version we've processed up to. So in short you need to do the following.

Get a GCS bucket for the blob store.

Get a VM in Compute Engine. Make sure to get one with a GPU (I'm using the cheapest one with a NVIDIA T4). The GCP UI will suggest an image that already has the GPU drivers installed, use that. If you don't, follow this guide: https://cloud.google.com/compute/docs/gpus/install-drivers-gpu.

Make sure to give the service account the VM is using the permission to write to GCS. I also gave it the roles/storage.objectAdmin scope.

Install rustup, build-essential, etc. Everything needed to build the binary. Install X11 and friends. Install postgres. Something like this should do it:
```
sudo apt update
sudo apt install -y ca-certificates curl gnupg xvfb x11-utils libxcursor1 libxinerama1 libgl1-mesa-glx libxkbcommon-x11-0 clang pkg-config libx11-dev libudev-dev xserver-xorg-video-dummy x11-xserver-utils postgresql postgresql-client
```

Clone this repo on the VM, cd in to `art/`, and run `cargo build -p processor --release`.

Crate a database and set a user password:
```
sudo -u postgres psql -c "CREATE DATABASE summits;"
sudo -u postgres psql -c "ALTER USER postgres WITH PASSWORD 'blah';"
```

Create a config for the processor like this at `/home/dport/aptos-summits/config.yaml`:
```
processor_config:
  stream_subscriber_config:
    grpc_data_service_address: https://grpc.mainnet.aptoslabs.com
    auth_token: token
  dispatcher_config: {}
  common_storage_config:
    initial_starting_version: 421207730
  processor_config::q
    contract_address: "0x67a614e8df22d397b7a7057d743e6b30f8ef2820c054a391658c06199187fa3c"
storage_config:
  connection_string: "postgres://postgres:blah@127.0.0.1:5432/summits"
blob_store_config:
  type: "Gcs"
  bucket_name: "aptos-summits"
bevy_width: 2000
```

I store the private key for the module deployer in GCP Secrets Manager.

Create a run script like this at `/home/dport/run.sh`:
```
#!/bin/bash

set -e
set -x

NUM=97

# Start Xvfb
Xvfb -ac :$NUM -screen 0 4096x4096x24 &
XVFB_PID=$!

# Ensure Xvfb is killed on script exit
trap "kill $XVFB_PID" EXIT

# Set the DISPLAY environment variable
export DISPLAY=:$NUM

# Run your binary command here
export WINIT_UNIX_BACKEND=x11
/home/dport/aptos-summits/art/target/release/processor -c /home/dport/config.yaml
```

If you have issues with this script try changing from 99 to some other number.

Create a systemd file like this at `/etc/systemd/system/processor.service`:
```
[Unit]
Description=Summits Processor
Wants=network.target
After=network-online.target

[Service]
Restart=always
TimeoutStopSec=5

# We assume that the binary has already been built.
ExecStart=/home/dport/run.sh

[Install]
WantedBy=multi-user.target default.target
```

Run and enable it:
```
sudo systemctl daemon-reload && sudo systemctl enable processor.service && sudo systemctl start processor.service
```

See how it's doing like this:
```
sudo journalctl -u processor -f
```

To reset the processor, do this:
```
sudo systemctl stop processor && sudo -u postgres psql -c "DROP DATABASE summits;" && sudo -u postgres psql -c "CREATE DATABASE summits;"
```
