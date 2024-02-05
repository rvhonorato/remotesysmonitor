# monitor

This is a small command-line utility that runs some checks on remote servers and posts the results to a Slack channel.

## Installation

```bash
git clone https://github.com/rvhonorato/monitor.git && cd monitor
cargo install --path .
```

## Usage

```bash
SLACK_HOOK_URL=<your-slack-hook-url> monitor configuration.yaml
```

You need to define `SLACK_HOOK_URL` as an environment variable with the URL of the Slack webhook you want to use.

The only argument is the path to a YAML file with the configuration. The file should look like this:

```yaml
servers:
  - name: name-of-your-server
    host: myserver.nl
    port: 22
    user: rodrigo
    private_key: /home/rodrigo/.ssh/id_ed25519
    checks:
      ping:
        url:
          - /
      load:
        interval: 15
      number_of_subfolders:
        path:
          - /path/full/of/subfolders
      custom_command:
        command: cd /some/path && ./some_script.sh
      list_old_directories:
        loc: /path/full/of/old/directories
        cutoff: 2 # days

  - name: raspeberryPi
    host: ip.of.your.raspberry.pi
    port: 22
    user: rodrigo
    private_key: /home/rodrigo/.ssh/id_ed25519
    checks:
      temperature:
        sensor: /sys/bus/w1/devices/28-00000b694311/w1_slave
```
