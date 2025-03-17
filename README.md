# Preload

A system module that hooks into the operating system's `execve` function to automatically load APM (Application Performance Monitoring) agents.

Key Features:

- Automatically detects the programming language of target applications
- Injects necessary environment variables based on the detected language-specific APM agent
- Auto-generates ServiceName (currently Java only)
- Maintains seamless program execution flow

## Installation

- Run `make dist` to create installation files, then execute `bash install.sh` for local installation
- Run `make release` to create `install-apo-instrument.tar.gz` package for deployment on other machines

## Uninstallation

Remove the following line from `/etc/ld.so.preload`:
```
/etc/apo/instrument/libapolanucher.so
```
If this is the only line in the file, you can safely remove the entire `/etc/ld.so.preload` file.
Restart your terminal session to complete the uninstallation.

**Troubleshooting**

If SSH access becomes unavailable due to preload issues, you can fix it by either:
- Using `scp` to copy an empty file to overwrite `/etc/ld.so.preload` on the target machine
- Completing the uninstallation process

To clean up remaining files afterward:
```
rm -r /etc/apo
```

## Usage in Virtual Machines

The preload hook takes effect for all program executions immediately after running `install.sh`.

## Usage in Docker Containers

After installing on the host machine, add these parameters when starting containers:

```
-v /etc/apo:/etc/apo
-e LD_PRELOAD=/etc/apo/instrument/libapolanucher.so
```

The `-v` flag mounts the agent files, while `-e` enables the preload library.

Example:

Original docker command:
```
docker run -d exampleApp:tag
```

Modified command:
```
docker run -d \
    -v /etc/apo:/etc/apo \
    -e LD_PRELOAD=/etc/apo/instrument/libapolanucher.so \
    exampleApp:tag
```
