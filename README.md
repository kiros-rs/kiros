# kiros-rs
KIROS is a collection of cross-platform modules designed to empower developers to build modular, cross-platform robotics systems for use in disaster situations. KIROS is based on my experiences from developing & deploying [RoboHUD](https://github.com/CCGSRobotics/RoboHUD) at the RoboCup RMRC competition in 2019.  

# Installation
As the project in its current form is quite barebones (and under heavy active development), it is fairly difficult to utilise it effectively in production. If you would like to check it out, please follow these steps:
- `git clone https://github.com/kiros-rs/kiros`
- `cd kiros`
- Install [just](https://github.com/casey/just)
- `just install-toolchain`
- `just build`

# Building the project
To compile the project, simply run `just build`, followed by the targets you wish to build for (default is local machine). For example, `just build linux windows` builds for linux & windows, while `just build` will run for the local machine. When compiling for a target, the script also installs the required Rust toolchains for you! Here are all the currently supported targets:
- `all`
- `linux`
- `windows`
- `mac`
- `rpi` - Raspberry Pi model 2/3/4
- `rpi-legacy` - Raspberry Pi model 0/1

# Examples
Soon....

# Contributing
As previously mentioned, the project is currently in its infancy. If you would like to discuss anything, please feel free to reach out to me on Discord `@Finchie#9461` or contact me via my email listed on GitHub.

Any commits should be first linted using `just lint` - this should soon be integrated into CI and added as a pre-commit hook, but for now please follow this process until I set the new system up.
