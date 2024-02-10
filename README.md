# monitor

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/a01b6cdccbe646eaa3afff5323358985)](https://app.codacy.com/gh/rvhonorato/monitor/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
[![test](https://github.com/rvhonorato/monitor/actions/workflows/test.yml/badge.svg)](https://github.com/rvhonorato/monitor/actions/workflows/test.yml)

This is a small command-line utility that runs some checks on remote servers and posts the results to a Slack channel.

[Check the documentation for a comprehensive explanation](https://www.rvhonorato.me/monitor/monitor/index.html) of how this tool works and also for a [full list of checks](https://www.rvhonorato.me/monitor/monitor/checks/index.html).

## TL:DR

### Install

```bash
git clone https://github.com/rvhonorato/monitor.git && cd monitor && \
  cargo install --path .
```

### Execute

Check the output of the `monitor --help` command for a list of available options.

```bash
$ monitor -h
Usage: monitor [OPTIONS] <CONFIG>

Arguments:
  <CONFIG>

Options:
  -f, --full     Post a check to Slack even if there is no ❌ in the checks
  -p, --print    Print the output of the checks in stdout
  -h, --help     Print help
  -V, --version  Print version
```

You need to define `SLACK_HOOK_URL` as an environment variable with the URL of the Slack webhook you want to use and a path to the configuration file.

```bash
SLACK_HOOK_URL=<your-slack-hook-url> monitor configuration.yaml
```

The configuration file should look like this:

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

It might make sense to configure a cron job to run this command periodically.

```bash
# Run every 10 minutes, it will only post to Slack if one of the checks has ❌
*/10 * * * * SLACK_HOOK_URL=<your-slack-hook-url> monitor configuration.yaml

# Post a full report to Slack at 8, 12, 16 and 20 hours
## Running with -f will post to Slack even if there is no ❌ in the checks
0 8,12,16,20 * * * SLACK_HOOK_URL=<your-slack-hook-url> monitor -f configuration.yaml
```


## Development

There is a `.devcontainer` configuration for VSCode, so you can use it to develop the project. It will setup a development environment and also configure a SSH server to test the checks that require a remote server.

Once inside the dev-container you can tweak the `conf/conf.dev.yaml` file to your needs and run the project with:

```
SLACK_HOOK_URL="" cargo run -- -p conf/conf.dev.yaml
```

Test it with:

```
cargo test
```