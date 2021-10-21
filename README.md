# Runner

Lightweight launcher and monitoring tool for projects and scripts

## Building

### Prerequisites

Download this repository:

```bash
git clone https://gitlab.com/everdream/runner
```

Install rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

All options can be left at default. You might need to restart your terminal at this point.

### Build

To test application, move your console into application folder and run:

```bash
cargo test
```

To build application in release mode, run

```bash
cargo build --release
```

Standalone executable will be available in `target/release` It can be moved from there to a convenient location.

## Configuration

Runner takes a json formatted configuration file. A file should look something like one below:

```json
{
    "application": "runner",
    "version": "0.3.2",
    "crash path": "./err",
    "commands": [
        {
            "command": "./updater/updater",
            "args": [ "-all" ],
            "mode": "run until success",
            "stdout history": 100,
            "name": "awesome updater"
        },
        {
            "command": "./application/app",
            "args": [ "-c", "custom_config.cfg" ],
            "mode": "keep alive"
        },
    ]
}
```
### Options

`application` field is mandatory and must be set to `"runner"`

`version` field is mandatory and must mach the version of runner you are using.

`crash path` field is mandatory. It is a location of folder where crash logs will be stored. Runner will create a folder if it doesn't exist, but will not work if it can not write to the location.

`commands` mandatory array of command configurations to be run. Runner will execute them in an order provided.

### Command configuration

`command` mandatory command to execute. If path to application it can be either relative or absolute.

`args` Optional array of strings to be passed to the command defined in command.

`mode` Optional mode to run application in. There are 5 possible values:

 * `"run once"` - Runs command once and stores any error logs but does not restart it
 * `"run once and wait"` - Same as run once, but waits for command to exit before continuing down the list
 * `"run until success"` - Restart application if it crashes / exits with non 0 status, but stops its execution once it exits with 0 status.
 * `"run until success and wait"` - same as run until success but waits for command to exit before continuing down the list
 * `"keep alive"` - Always restarts an application, keeping it alive as long as runner is alive.

 Default is `"run until success"`

 `stdout history` Number of lines of stdout to store in case of the crash. Larger numbers take more memory but can be useful when debugging any crashes that occurred. Default is 1000.

 `name` Runner tries to provide meaningful names for running processes from command field. However in cases where multiple python scripts are being run all of them will be shown as python. Name is optional field with a custom name to be shown both in tabs and as name in error folders.

 ## Running

 Once we have desired configuration file ready, runner can be started with

 ```bash
 runner -c config.json
 ```

 where `config.json` is a path to configuration file we have created.

 Runner is very lightweight and heavily multithreaded. On some systems and configurations the default number of threads might be too small to support all monitoring jobs required. If you encounter any problems such as tui freezing, try to set environmental variable `ASYNC_STD_THREAD_COUNT` to a bigger positive numbers.

