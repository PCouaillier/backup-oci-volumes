# backup oci volumes

## What is the purpose ?

This tools allows to backup an OCI (docker/podman) volume as a `.tar.gz`.

The tools use user/password or a rsa `.pem` file to connect to a remote host and export volumes locally.

## Usage

Backuping to a directory:

```
backup-oci-volumes \
    -h target-host \
    -p 2222 \
    -u podman-user \
    -t "./host-$(date '+%Y-%m-%dT%H:%M:%S')-volume/"
```

The exmple uses `~/.ssh/id_rsa.pem` key. If you need user/password use `-P`.
